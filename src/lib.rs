#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
//! A Pretty Darn Fast library for creating PDF files.
//! Currently only supports basic images, simple vector graphics, and text with builtin fonts (but not UTF-8).
//!

//! # Example
//!
//! ```rust
//! use pdfpdf::{Color, Pdf, Point, Size, Alignment};
//!
//! Pdf::new()
//!     .add_page(Size { x: 180.0, y: 240.0 })
//!     .set_color(Color::rgb(0, 0, 248))
//!     .draw_circle(Point{ x: 90.0, y: 120.0 }, 50.0)
//!     .write_to("example.pdf")
//!     .unwrap();
//! ```
//!
//! To use this library you need to add it as a dependency in your
//! `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! pdfpdf = "0.3"
//! ```
//!
//! More working examples can be found in [here](https://github.com/saethlin/pdfpdf/tree/master/examples).
#![warn(missing_docs)]

use std::fs::File;
use std::io;

mod fonts;
mod graphicsstate;
mod image;
mod text;
#[macro_use]
mod util;

pub use fonts::Font;
pub use graphicsstate::{Color, Matrix};
pub use image::Image;
pub use text::Alignment;

use util::Formattable;
pub use util::{Point, Size};

/// Available compression levels for a PDF document's internal streams
/// This is configurable on a per-page basis
#[derive(Clone, Copy, Debug)]
pub enum Compression {
    /// Fast is a good default, the compression is surprisingly good
    Fast,
    /// Normal is deflate's default
    Normal,
    /// Best deflate compression available, sacrifices a lot of runtime for not much size
    Best,
    /// Uncompressed PDF streams are both easier to debug and much faster to write.
    /// Some uncompressed PDFs may be slower due to the amount of disk reads required.
    Off,
}

impl Compression {
    fn to_deflate(self) -> Option<deflate::Compression> {
        match self {
            Compression::Fast => Some(deflate::Compression::Fast),
            Compression::Normal => Some(deflate::Compression::Default),
            Compression::Best => Some(deflate::Compression::Best),
            Compression::Off => None,
        }
    }
}

/// Represents a PDF internal object
struct PdfObject {
    contents: Vec<u8>,
    id: usize,
    is_page: bool,
    is_xobject: bool,
    offset: Option<usize>,
}

/// The top-level struct that represents a (partially) in-memory PDF file
pub struct Pdf {
    buffer: Vec<u8>,
    page_buffer: Vec<u8>,
    objects: Vec<PdfObject>,
    width: f64,
    height: f64,
    fonts: Vec<fonts::Font>,
    font_size: f64,
    current_font_index: usize,
    compression: Compression,
}

impl Default for Pdf {
    fn default() -> Self {
        Self::new()
    }
}

impl Pdf {
    /// Create a new blank PDF document
    #[inline]
    pub fn new() -> Self {
        Self {
            buffer: b"%PDF-1.7\n%\xB5\xED\xAE\xFB\n".to_vec(),
            page_buffer: Vec::new(),
            objects: vec![
                PdfObject {
                    contents: Vec::new(),
                    id: 1,
                    is_page: false,
                    is_xobject: false,
                    offset: None,
                },
                PdfObject {
                    contents: Vec::new(),
                    id: 2,
                    is_page: false,
                    is_xobject: false,
                    offset: None,
                },
            ],
            width: 400.0,
            height: 400.0,
            fonts: vec![Font::Helvetica],
            font_size: 12.0,
            current_font_index: 0,
            compression: Compression::Fast,
        }
    }

    fn add_object(&mut self, data: Vec<u8>, is_page: bool, is_xobject: bool) -> usize {
        let id = self.objects.iter().map(|o| o.id).max().unwrap_or(3) + 1;
        self.objects.push(PdfObject {
            contents: data,
            id,
            is_page,
            is_xobject,
            offset: None,
        });
        id
    }

    /// Sets the compression level for this document
    /// Calls to this method do not affect data produced by operations before the last .add_page
    #[inline]
    pub fn compression(&mut self, compression: Compression) -> &mut Self {
        self.compression = compression;
        self
    }

    /// Set the PDF clipping box for the current page
    #[inline]
    pub fn set_clipping_box<X, Y, W, H>(
        &mut self,
        location: Point<X, Y>,
        size: Size<W, H>,
    ) -> &mut Self
    where
        X: Into<f64>,
        Y: Into<f64>,
        W: Into<f64>,
        H: Into<f64>,
    {
        let corner = location.into_f64();
        let size = size.into_f64();

        ryu!(
            self.page_buffer,
            corner.x,
            corner.y,
            size.width,
            size.height,
            "re W n" // W uses nonzero winding rule
        );
        self
    }

    /// Add an RGB image
    #[inline]
    pub fn add_image_at<X, Y>(&mut self, image: Image, location: Point<X, Y>) -> &mut Self
    where
        X: Into<f64>,
        Y: Into<f64>,
    {
        use deflate::{deflate_bytes_zlib_conf, Compression};
        use std::io::Write;

        let location = location.into_f64();

        let compressed = deflate_bytes_zlib_conf(image.buf, Compression::Best);

        let _ = write!(
            self.page_buffer,
            "q {} 0 0 {} {} {} cm\n\
             BI\n\
             /W {}\n\
             /H {}\n\
             /CS /RGB\n\
             /BPC 8\n\
             /F [/Fl]\n\
             ID\n",
            image.width, image.height, location.x, location.y, image.width, image.height
        );
        self.page_buffer.extend(compressed);
        self.page_buffer.extend(b"\nEI Q\n");

        self
    }

    /// Move the pen, starting a new path
    #[inline]
    pub fn move_to<X, Y>(&mut self, p: Point<X, Y>) -> &mut Self
    where
        Y: Into<f64>,
        X: Into<f64>,
    {
        let p = p.into_f64();
        ryu!(self.page_buffer, p.x, p.y, "m");
        self
    }

    /// Draw a line from the current location
    #[inline]
    pub fn line_to<X, Y>(&mut self, p: Point<X, Y>) -> &mut Self
    where
        Y: Into<f64>,
        X: Into<f64>,
    {
        let p = p.into_f64();
        ryu!(self.page_buffer, p.x, p.y, "l");
        self
    }

    /// Draw a cubic Bézier curve
    #[inline]
    pub fn curve_to(
        &mut self,
        (x1, y1): (f64, f64),
        (x2, y2): (f64, f64),
        (x3, y3): (f64, f64),
    ) -> &mut Self {
        ryu!(&mut self.page_buffer, x1, y1, x2, y2, x3, y3, "c");
        self
    }

    /// Set the current line width
    #[inline]
    pub fn set_line_width<N>(&mut self, width: N) -> &mut Self
    where
        N: Into<f64>,
    {
        ryu!(self.page_buffer, width.into(), "w");
        self
    }

    /// Set the color for all subsequent drawing operations
    #[inline]
    pub fn set_color(&mut self, color: Color) -> &mut Self {
        let norm = |color| f64::from(color) / 255.0;
        ryu!(
            self.page_buffer,
            norm(color.red),
            norm(color.green),
            norm(color.blue),
            "SC"
        );
        ryu!(
            self.page_buffer,
            norm(color.red),
            norm(color.green),
            norm(color.blue),
            "rg"
        );
        self
    }

    /// Apply a coordinate transformation to all subsequent drawing calls
    /// Consecutive applications of this function are cumulative
    #[inline]
    pub fn transform(&mut self, m: Matrix) -> &mut Self {
        self.page_buffer.extend(format!("{} cm\n", m).bytes());
        self
    }

    /// Draw a circle with the current drawing configuration,
    /// based on http://spencermortensen.com/articles/bezier-circle/
    #[inline]
    pub fn draw_circle<X, Y, N>(&mut self, center: Point<X, Y>, radius: N) -> &mut Self
    where
        Y: Into<f64>,
        X: Into<f64>,
        N: Into<f64>,
    {
        let center = center.into_f64();
        let radius = radius.into();
        let x = center.x;
        let y = center.y;
        let top = y - radius;
        let bottom = y + radius;
        let left = x - radius;
        let right = x + radius;
        let c = 0.551_915_024_494;
        let leftp = x - (radius * c);
        let rightp = x + (radius * c);
        let topp = y - (radius * c);
        let bottomp = y + (radius * c);
        self.move_to(Point { x, y: top });
        self.curve_to((leftp, top), (left, topp), (left, y));
        self.curve_to((left, bottomp), (leftp, bottom), (x, bottom));
        self.curve_to((rightp, bottom), (right, bottomp), (right, y));
        self.curve_to((right, topp), (rightp, top), (x, top));
        self.page_buffer.extend(b"S\n"); // close and stroke
        self
    }

    /// Draw a circle with the current drawing configuration,
    /// based on http://spencermortensen.com/articles/bezier-circle/
    #[inline]
    pub fn draw_circle_filled<X, Y, N>(&mut self, center: Point<X, Y>, radius: N) -> &mut Self
    where
        Y: Into<f64>,
        X: Into<f64>,
        N: Into<f64>,
    {
        let center = center.into_f64();
        let x = center.x;
        let y = center.y;
        let radius = radius.into();
        let top = y - radius;
        let bottom = y + radius;
        let left = x - radius;
        let right = x + radius;
        let c = 0.551_915_024_494;
        let leftp = x - (radius * c);
        let rightp = x + (radius * c);
        let topp = y - (radius * c);
        let bottomp = y + (radius * c);
        self.move_to(Point { x, y: top });
        self.curve_to((leftp, top), (left, topp), (left, y));
        self.curve_to((left, bottomp), (leftp, bottom), (x, bottom));
        self.curve_to((rightp, bottom), (right, bottomp), (right, y));
        self.curve_to((right, topp), (rightp, top), (x, top));
        self.page_buffer.extend(b"f\n"); // implicitly close and fill
        self
    }

    // TODO: This should actually be something like a
    // let id = pdf.draw_xobject
    /// Draw multiple dots using an XObject to save space
    #[inline]
    pub fn draw_dots(&mut self, x: &[f64], y: &[f64]) -> &mut Self {
        let c = 0.551_915_024_494;
        let mut dot = Vec::new();
        ryu!(dot, 0., -1., "m");
        ryu!(dot, -c, -1., -1., -c, -1., 0., "c");
        ryu!(dot, -1., c, -c, 1., 0., 1., "c");
        ryu!(dot, c, 1., 1., c, 1., 0., "c");
        ryu!(dot, 1., -c, c, -1., 0., -1., "c", "f");
        let mut dot_obj = format!(
            "<< /Type /XObject /Subtype /Form /BBox [ -2 -2 2 2 ] /Length {} >>\nstream\n",
            dot.len()
        )
        .into_bytes();
        dot_obj.extend_from_slice(&dot);
        dot_obj.extend_from_slice(b"endstream\n");

        let id = self.add_object(dot_obj, false, false);

        self.add_object(format!("<< /M0 {} 0 R >>\n", id).into_bytes(), false, true);

        self.page_buffer.extend(b"q\n");
        let mut previous = Point { x: 0.0, y: 0.0 };
        for (x, y) in x.iter().zip(y) {
            ryu!(
                self.page_buffer,
                1.,
                0.,
                0.,
                1.,
                x - previous.x,
                y - previous.y,
                "cm /M0 Do"
            );
            previous.x = *x;
            previous.y = *y;
        }
        self.page_buffer.extend(b"Q\n");

        self
    }

    /// Draw a line between all these points in the order they appear
    #[inline]
    pub fn draw_line<I1, I2>(&mut self, x_iter: I1, y_iter: I2) -> &mut Self
    where
        I1: IntoIterator<Item = f64>,
        I2: IntoIterator<Item = f64>,
    {
        let mut x_iter = x_iter.into_iter();
        let mut y_iter = y_iter.into_iter();
        // Can't just loop because we have to move_to the first point, then we can line_to the rest
        if let (Some(x), Some(y)) = (x_iter.next(), y_iter.next()) {
            self.move_to(Point { x, y });
            for (x, y) in x_iter.zip(y_iter) {
                self.line_to(Point { x, y });
            }
        }
        self.page_buffer.extend(b"S\n");
        self
    }

    /// End a line
    #[inline]
    pub fn end_line(&mut self) -> &mut Self {
        self.page_buffer.extend(b"S\n");
        self
    }

    /// Draw a rectangle in the current color with bottom-left corner at with bottom-lef
    /// corner at `corner` and dimensions `size`.

    #[inline]
    pub fn draw_rectangle_filled<X, Y, W, H>(
        &mut self,
        corner: Point<X, Y>,
        size: Size<W, H>,
    ) -> &mut Self
    where
        X: Into<f64>,
        Y: Into<f64>,
        W: Into<f64>,
        H: Into<f64>,
    {
        let corner = corner.into_f64();
        let size = size.into_f64();
        ryu!(
            self.page_buffer,
            corner.x,
            corner.y,
            size.width,
            size.height,
            "re f" // Fill path using Nonzero Winding Number Rule
        );
        self
    }

    /// Draw a shaded rectangle in the current color with bottom-left corner at with bottom-left
    /// corner at `corner` and dimensions `size`.
    #[inline]
    pub fn draw_rectangle<X, Y, W, H>(&mut self, corner: Point<X, Y>, size: Size<W, H>) -> &mut Self
    where
        X: Into<f64>,
        Y: Into<f64>,
        W: Into<f64>,
        H: Into<f64>,
    {
        let corner = corner.into_f64();
        let size = size.into_f64();

        ryu!(
            self.page_buffer,
            corner.x,
            corner.y,
            size.width,
            size.height,
            "re S" // Fill path using Nonzero Winding Number Rule
        );
        self
    }

    /// Set the font for all subsequent drawing calls
    #[inline]
    pub fn font<N>(&mut self, font: Font, size: N) -> &mut Self
    where
        N: Into<f64>,
    {
        match self.fonts.iter().position(|f| *f == font) {
            Some(index) => {
                self.current_font_index = index;
            }
            None => {
                self.fonts.push(font);
                self.current_font_index = self.fonts.len() - 1;
            }
        }
        self.font_size = size.into();
        self
    }

    /// Convienence method to figure out the width of a string
    /// May be required for some users to position text properly
    pub fn width_of(&self, text: &str) -> f64 {
        let current_font = &self.fonts[self.current_font_index];
        text.chars()
            .filter(|c| *c != '\n')
            .map(|c| fonts::glyph_width(current_font, c))
            .sum::<f64>()
            * self.font_size
    }

    /// Draw text at a given location with the current settings
    #[inline]
    pub fn draw_text<X, Y>(
        &mut self,
        position: Point<X, Y>,
        alignment: Alignment,
        text: &str,
    ) -> &mut Self
    where
        X: Into<f64>,
        Y: Into<f64>,
    {
        let x = position.x.into();
        let y = position.y.into();
        let height = self.font_size;

        self.page_buffer
            .extend(format!("BT\n/F{} {} Tf\n", self.current_font_index, self.font_size).bytes());

        let num_lines = text.split('\n').count() as f64;
        for (l, line) in text.split('\n').enumerate() {
            let line_width = self.width_of(line);
            let l = l as f64;

            let (line_x, line_y) = match alignment {
                Alignment::TopLeft => (x, y - height * (l + 1.0)),
                Alignment::TopRight => (x - line_width, y - height * (l + 1.0)),
                Alignment::TopCenter => (x - line_width / 2.0, y - height * (l + 1.0)),
                Alignment::CenterLeft => (
                    x,
                    (y - height / 3.0) - (l - (num_lines - 1.0) / 2.0) * height * 1.25,
                ),
                Alignment::CenterRight => (
                    x - line_width,
                    (y - height / 3.0) - (l - (num_lines - 1.0) / 2.0) * height * 1.25,
                ),
                Alignment::CenterCenter => (
                    x - line_width / 2.0,
                    (y - height / 3.0) - (l - (num_lines - 1.0) / 2.0) * height * 1.25,
                ),
                Alignment::BottomLeft => (x, y + (num_lines - l - 1.0) * 1.25 * height),
                Alignment::BottomRight => {
                    (x - line_width, y + (num_lines - l - 1.0) * 1.25 * height)
                }
                Alignment::BottomCenter => (
                    x - line_width / 2.0,
                    y + (num_lines - l - 1.0) * 1.25 * height,
                ),
            };

            self.page_buffer
                .extend(format!("1 0 0 1 {} {} Tm (", line_x, line_y).bytes());
            for c in line.chars() {
                let data = format!("\\{:o}", c as u32);
                self.page_buffer.extend(data.bytes());
            }
            self.page_buffer.extend(b") Tj\n");
        }
        self.page_buffer.extend(b"ET\n");
        self
    }

    /// Move to a new page in the PDF document
    #[inline]
    pub fn add_page<W, H>(&mut self, size: Size<W, H>) -> &mut Self
    where
        W: Into<f64>,
        H: Into<f64>,
    {
        // Compress and write out the previous page if it exists
        if !self.page_buffer.is_empty() {
            self.end_page();
            self.page_buffer.clear();
        }

        self.page_buffer
            .extend("/DeviceRGB cs /DeviceRGB CS\n1 j 1 J\n".bytes());
        self.width = size.width.into();
        self.height = size.height.into();
        self
    }

    /// Dump a page out to disk
    fn end_page(&mut self) {
        // Write out any images associated with this page
        // TODO: are images global or associated with a page?

        let page_stream = if let Some(level) = self.compression.to_deflate() {
            let compressed = deflate::deflate_bytes_zlib_conf(&self.page_buffer, level);
            let mut page = format!(
                "<< /Length {} /Filter [/FlateDecode] >>\nstream\n",
                compressed.len()
            )
            .into_bytes();
            page.extend_from_slice(&compressed);
            page.extend(b"endstream\n");
            page
        } else {
            let mut page = Vec::new();
            page.extend(format!("<< /Length {} >>\nstream\n", self.page_buffer.len()).bytes());
            page.extend(&self.page_buffer);
            page.extend(b"endstream\n");
            page
        };

        // Create the stream object for this page
        let stream_object_id = self.add_object(page_stream, false, false);

        // Create the page object, which describes settings for the whole page
        let mut page_object = b"<< /Type /Page\n \
            /Parent 2 0 R\n \
            /Resources <<\n"
            .to_vec();

        for obj in self.objects.iter().filter(|o| o.is_xobject) {
            page_object.extend(format!("/XObject {} 0 R ", obj.id).bytes());
        }

        for (f, font) in self.fonts.iter().enumerate() {
            page_object.extend(
                format!(
                    "  /Font <<\n   /F{} <<\n    /Type /Font\n    /Subtype /Type1\n    /BaseFont \
                     /{:?}\n    /Encoding /WinAnsiEncoding\n   >>\n  >>\n",
                    f, font
                )
                .bytes(),
            );
        }
        page_object.extend_from_slice(
            format!(
                " >>\n \
                 /MediaBox [0 0 {} {}]\n \
                 /Contents {} 0 R\n\
                 >>\n",
                self.width, self.height, stream_object_id
            )
            .as_bytes(),
        );
        self.add_object(page_object, true, false);

        self.fonts.truncate(1);
    }

    /// Write the in-memory PDF representation to disk
    pub fn write_to(&mut self, filename: &str) -> io::Result<()> {
        use std::io::Write;

        if !self.page_buffer.is_empty() {
            self.end_page();
        }

        // Write out each object
        for obj in self.objects.iter_mut().skip(2) {
            obj.offset = Some(self.buffer.len());
            self.buffer.extend(format!("{} 0 obj\n", obj.id).as_bytes());
            self.buffer.extend_from_slice(&obj.contents);
            self.buffer.extend_from_slice(b"endobj\n");
        }

        // Write out the page tree object
        self.objects[1].offset = Some(self.buffer.len());
        self.buffer.extend(b"2 0 obj\n");
        self.buffer.extend(b"<< /Type /Pages\n");
        self.buffer.extend(
            format!(
                "/Count {}\n",
                self.objects.iter().filter(|o| o.is_page).count()
            )
            .bytes(),
        );
        self.buffer.extend(b"/Kids [");
        for obj in self.objects.iter().filter(|obj| obj.is_page) {
            self.buffer.extend(format!("{} 0 R ", obj.id).bytes());
        }
        self.buffer.pop();
        self.buffer.extend(b"] >>\nendobj\n");

        // Write out the catalog dictionary object
        self.objects[0].offset = Some(self.buffer.len());
        self.buffer
            .extend_from_slice(b"1 0 obj\n<< /Type /Catalog\n/Pages 2 0 R >>\nendobj\n");

        // Write the cross-reference table
        let startxref = self.buffer.len() + 1; // NOTE: apparently there's some 1-based indexing??
        self.buffer.extend(b"xref\n");
        self.buffer
            .extend(format!("0 {}\n", self.objects.len() + 1).bytes());
        self.buffer.extend(b"0000000000 65535 f \n");
        self.objects.sort_by(|a, b| a.id.cmp(&b.id));

        for obj in &self.objects {
            self.buffer
                .extend(format!("{:010} 00000 f \n", obj.offset.unwrap()).bytes());
        }

        // Write the document trailer
        self.buffer.extend(b"trailer\n");
        self.buffer
            .extend(format!("<< /Size {}\n", self.objects.len()).bytes());
        self.buffer.extend(b"/Root 1 0 R >>\n");

        // Write the offset to the xref table
        self.buffer
            .extend(format!("startxref\n{}\n", startxref).bytes());

        // Write the PDF EOF
        self.buffer.extend(b"%%EOF");

        File::create(filename)?.write_all(self.buffer.as_slice())
    }
}
