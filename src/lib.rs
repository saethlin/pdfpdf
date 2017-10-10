//! A Pretty Darn Fast library for creating PDF files. Currently, only simple vector graphics and simple text are supported.
//!
//!

//! # Example
//!
//! ```
//! use pdfpdf::Pdf;
//! use pdfpdf::graphicsstate::Color;
//!
//! Pdf::new()
//!     .add_page(180.0, 240.0)
//!     .set_color(&Color::rgb(0, 0, 248))
//!     .draw_circle(90.0, 120.0, 50.0)
//!     .write_to("example.pdf")
//!     .expect("Failed to write to file");
//! ```
//!
//! To use this library you need to add it as a dependency in your
//! `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! pdfpdf = "0.2"
//! ```
//!
//! More working examples can be found in [here]
//! (https://github.com/saethlin/pdfpdf/tree/master/examples).
#![deny(missing_docs)]

#[macro_use]
extern crate lazy_static;
extern crate deflate;
extern crate num;

use num::NumCast;
use std::fs::File;
use std::io;

mod graphicsstate;
mod fonts;
mod text;
pub use fonts::Font;
pub use graphicsstate::{Color, Matrix};
pub use text::Alignment;

// Represents a PDF internal object
struct PdfObject {
    offset: usize,
    id: usize,
    is_page: bool,
}

/// The top-level struct that represents an in-memory PDF file
pub struct Pdf {
    buffer: Vec<u8>,
    page_buffer: Vec<u8>,
    objects: Vec<PdfObject>,
    width: f64,
    height: f64,
    font: fonts::Font,
    font_size: f64,
    compress: bool,
}

impl Pdf {
    /// Create a new blank PDF document
    #[inline]
    pub fn new() -> Self {
        let mut buffer = Vec::new();
        buffer.extend(b"%PDF-1.7\n%\xB5\xED\xAE\xFB\n");
        Pdf {
            buffer: buffer,
            page_buffer: Vec::new(),
            objects: vec![
                PdfObject {
                    offset: 0,
                    id: 1,
                    is_page: false,
                },
                PdfObject {
                    offset: 0,
                    id: 2,
                    is_page: false,
                },
            ],
            width: 400.0,
            height: 400.0,
            font: fonts::Font::Helvetica,
            font_size: 12.0,
            compress: true,
        }
    }

    /// Create a new blank PDF document without compression
    #[inline]
    pub fn new_uncompressed() -> Self {
        let mut this = Pdf::new();
        this.compress = false;
        this
    }

    /// Move then pen, starting a new path
    #[inline]
    fn move_to(&mut self, x: f64, y: f64) -> &mut Self {
        self.page_buffer.extend(
            format!("{:.2} {:.2} m ", x, y).bytes(),
        );
        self
    }

    /// Draw a line from the current location
    #[inline]
    fn line_to(&mut self, x: f64, y: f64) -> &mut Self {
        self.page_buffer.extend(
            format!("{:.2} {:.2} l ", x, y).bytes(),
        );
        self
    }

    // Draw a cubic Bézier curve
    #[inline]
    fn curve_to(
        &mut self,
        (x1, y1): (f64, f64),
        (x2, y2): (f64, f64),
        (x3, y3): (f64, f64),
    ) -> &mut Self {
        self.page_buffer.extend(
            format!(
                "{:.2} {:.2} {:.2} {:.2} {:.2} {:.2} c\n",
                x1,
                y1,
                x2,
                y2,
                x3,
                y3
            ).bytes(),
        );
        self
    }

    /// Set the current line width
    #[inline]
    pub fn set_line_width<N: NumCast>(&mut self, width: N) -> &mut Self {
        self.page_buffer.extend(
            format!(
                "{:.2} w\n",
                width.to_f64().unwrap()
            ).bytes(),
        );
        self
    }

    /// Set the drawing color for the stroke operation,
    /// does not affect fill calls
    #[inline]
    pub fn set_color(&mut self, color: &Color) -> &mut Self {
        let norm = |color| color as f64 / 255.0;
        self.page_buffer.extend(
            format!(
                "{:.2} {:.2} {:.2} SC\n",
                norm(color.red),
                norm(color.green),
                norm(color.blue),
            ).bytes(),
        );
        self.page_buffer.extend(
            format!(
                "{:.2} {:.2} {:.2} rg\n",
                norm(color.red),
                norm(color.green),
                norm(color.blue),
            ).bytes(),
        );

        self
    }

    /// Apply a coordinate transformation to all subsequent drawing calls
    #[inline]
    pub fn transform(&mut self, m: Matrix) -> &mut Self {
        self.page_buffer.extend(format!("{} cm\n", m).bytes());
        self
    }

    /// Draw a circle with the current drawing configuration,
    /// based on http://spencermortensen.com/articles/bezier-circle/
    #[inline]
    pub fn draw_circle<N: NumCast>(&mut self, x: N, y: N, radius: N) -> &mut Self {
        let x = x.to_f64().unwrap();
        let y = y.to_f64().unwrap();
        let radius = radius.to_f64().unwrap();
        let top = y - radius;
        let bottom = y + radius;
        let left = x - radius;
        let right = x + radius;
        let c = 0.551915024494;
        let leftp = x - (radius * c);
        let rightp = x + (radius * c);
        let topp = y - (radius * c);
        let bottomp = y + (radius * c);
        self.move_to(x, top);
        self.curve_to((leftp, top), (left, topp), (left, y));
        self.curve_to((left, bottomp), (leftp, bottom), (x, bottom));
        self.curve_to((rightp, bottom), (right, bottomp), (right, y));
        self.curve_to((right, topp), (rightp, top), (x, top));
        self.page_buffer.extend_from_slice(b"S\n");
        self
    }

    /// Draw a line between all these points in the order they appear
    #[inline]
    pub fn draw_line<I, N: NumCast>(&mut self, mut points: I) -> &mut Self
    where
        I: Iterator<Item = (N, N)>,
    {
        if let Some((x, y)) = points.next() {
            let x = x.to_f64().unwrap();
            let y = y.to_f64().unwrap();
            self.move_to(x, y);
            for (x, y) in points {
                let x = x.to_f64().unwrap();
                let y = y.to_f64().unwrap();
                self.line_to(x, y);
            }
        }
        self.page_buffer.extend_from_slice(b"S\n");
        self
    }

    /// Draw a rectangle that extends from x1, y1 to x2, y2
    #[inline]
    pub fn draw_rectangle_filled<N: NumCast>(
        &mut self,
        x: N,
        y: N,
        width: N,
        height: N,
    ) -> &mut Self {
        self.page_buffer.extend(
            format!(
                "{:.2} {:.2} {:.2} {:.2} re\n",
                x.to_f64().unwrap(),
                y.to_f64().unwrap(),
                width.to_f64().unwrap(),
                height.to_f64().unwrap()
            ).bytes(),
        );
        // Fill path using Nonzero Winding Number Rule
        self.page_buffer.extend_from_slice(b"f\n");
        self
    }

    #[inline]
    /// Set the font for all subsequent drawing calls
    pub fn font<N: NumCast>(&mut self, font: Font, size: N) -> &mut Self {
        self.font = font;
        self.font_size = size.to_f64().unwrap();
        self
    }

    /// Draw text at a given location with the current settings
    #[inline]
    pub fn draw_text<N: NumCast>(
        &mut self,
        x: N,
        y: N,
        alignment: Alignment,
        text: &str,
    ) -> &mut Self {

        let x = x.to_f64().unwrap();
        let y = y.to_f64().unwrap();
        let widths = &fonts::GLYPH_WIDTHS[&self.font];
        let height = self.font_size;

        self.page_buffer.extend(
            format!("BT\n/F13 {} Tf\n", self.font_size)
                .bytes(),
        );

        let num_lines = text.split('\n').count();
        for (l, line) in text.split('\n').enumerate() {
            let line_width = line.chars()
                .filter(|c| *c != '\n')
                .map(|c| *widths.get(&c).unwrap_or(&1.0))
                .sum::<f64>() * self.font_size;

            let (line_x, line_y) = match alignment {
                Alignment::TopLeft => (x, y - height),
                Alignment::TopRight => (x - line_width, y - height),
                Alignment::TopCenter => (x - line_width / 2.0, y - height),
                Alignment::BottomLeft => (x, y),
                Alignment::BottomRight => (x - line_width, y),
                Alignment::BottomCenter => (x - line_width / 2.0, y),
                Alignment::CenterCenter => (x - line_width / 2.0, (y - height / 3.0) - (l as f64 - (num_lines as f64 -1.0)/2.0) * height * 1.25),
            };

            self.page_buffer.extend(format!("1 0 0 1 {} {} Tm (", line_x, line_y).bytes());
            for c in line.chars() {
                let data = format!("\\{:o}", c as u32);
                self.page_buffer.extend(data.bytes());
            }
            self.page_buffer.extend(b") Tj\n");
        }
        self.page_buffer.extend(b"ET\n");
        self
    }

    // TODO: test with multi-page documents
    /// Move to a new page in the PDF document
    #[inline]
    pub fn add_page<N: NumCast>(&mut self, width: N, height: N) -> &mut Self {
        // Compress and write out the previous page if it exists
        if !self.page_buffer.is_empty() {
            self.flush_page();
        }

        self.page_buffer.extend(
            "/DeviceRGB cs /DeviceRGB CS\n".bytes(),
        );
        self.width = width.to_f64().unwrap();
        self.height = height.to_f64().unwrap();
        self
    }

    /// Dump a page out to disk
    fn flush_page(&mut self) {
        // Write out the data stream for this page
        let obj_id = self.objects.iter().map(|o| o.id).max().unwrap() + 1;
        self.objects.push(PdfObject {
            offset: self.buffer.len(),
            id: obj_id,
            is_page: false,
        });

        let mut compressed = self.page_buffer.clone();
        let mut rounds = 0;
        if self.compress {
            loop {
                let another = deflate::deflate_bytes_zlib(compressed.as_slice());
                if another.len() < compressed.len() {
                    compressed = another;
                    rounds += 1;
                } else {
                    break;
                }
            }
        }
        self.buffer.extend(format!("{} 0 obj\n", obj_id).bytes());
        self.buffer.extend(
            format!(
                "<</Length {}\n/Filter [{}]>>\nstream\n",
                compressed.len(),
                "/FlateDecode ".repeat(rounds)
            ).bytes(),
        );

        self.buffer.extend(compressed.iter());
        self.buffer.extend("endstream\nendobj\n".bytes());
        self.page_buffer.clear();

        // Write out the page object
        let obj_id = self.objects.iter().map(|o| o.id).max().unwrap() + 1;
        self.objects.push(PdfObject {
            offset: self.buffer.len(),
            id: obj_id,
            is_page: true,
        });
        self.buffer.extend(format!("{} 0 obj\n", obj_id).bytes());
        self.buffer.extend_from_slice(b"<</Type /Page\n");
        self.buffer.extend_from_slice(b"/Parent 2 0 R\n");
        // TODO: Temporary restricted fonts
        self.buffer.extend_from_slice(
            b"/Resources << /Font << /F13 << /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding>> >> >>\n",
        );
        self.buffer.extend(
            format!("/MediaBox [0 0 {} {}]\n", self.width, self.height).bytes(),
        );
        self.buffer.extend(
            format!("/Contents {} 0 R", obj_id - 1).bytes(),
        );
        self.buffer.extend_from_slice(b">>\nendobj\n");
    }

    /// Write the in-memory PDF representation to disk
    pub fn write_to(&mut self, filename: &str) -> io::Result<()> {
        use std::io::Write;

        if !self.page_buffer.is_empty() {
            self.flush_page();
        }

        // Write out the page tree object
        self.objects[1].offset = self.buffer.len();
        self.buffer.extend_from_slice(b"2 0 obj\n");
        self.buffer.extend_from_slice(b"<</Type /Pages\n");
        self.buffer.extend(
            format!(
                "/Count {}\n",
                self.objects.iter().filter(|o| o.is_page).count()
            ).bytes(),
        );
        self.buffer.extend_from_slice(b"/Kids [");
        for obj in self.objects.iter().filter(|obj| obj.is_page) {
            self.buffer.extend(format!("{} 0 R ", obj.id).bytes());
        }
        self.buffer.pop();
        self.buffer.extend_from_slice(b"]>>\nendobj\n");

        // Write out the catalog dictionary object
        self.objects[0].offset = self.buffer.len();
        self.buffer.extend_from_slice(
            b"1 0 obj\n<</Type /Catalog\n/Pages 2 0 R>>\nendobj\n",
        );

        // Write the cross-reference table
        let startxref = self.buffer.len();
        self.buffer.extend_from_slice(b"xref\n");
        self.buffer.extend(
            format!("0 {}\n", self.objects.len() + 1).bytes(),
        );
        self.buffer.extend_from_slice(b"0000000000 65535 f \n");
        self.objects.sort_by(|a, b| a.id.cmp(&b.id));

        for obj in &self.objects {
            self.buffer.extend(
                format!("{:010} 00000 f \n", obj.offset).bytes(),
            );
        }

        // Write the document trailer
        self.buffer.extend_from_slice(b"trailer\n");
        self.buffer.extend(
            format!("<</Size {}\n", self.objects.len())
                .bytes(),
        );
        self.buffer.extend_from_slice(b"/Root 1 0 R>>\n");

        // Write the offset to the xref table
        self.buffer.extend(
            format!("startxref\n{}\n", startxref).bytes(),
        );

        // Write the PDF EOF
        self.buffer.extend_from_slice(b"%%EOF");

        File::create(filename)?.write_all(self.buffer.as_slice())
    }
}
