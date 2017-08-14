//! A Pretty Darn Fast library for creating PDF files.
//!
//! Currently, only simple vector graphics are supported

//! # Example
//!
//! ```
//! use pdfpdf::Pdf;
//! use pdfpdf::graphicsstate::Color;
//!
//! Pdf::new()
//!     .add_page(180.0, 240.0)
//!     .set_stroke_color(Color::rgb(0, 0, 248))
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
//! pdfpdf = "*"
//! ```
//!
//! More working examples can be found in [here]
//! (https://github.com/saethlin/pdfpdf/tree/master/examples).
#![deny(missing_docs)]

extern crate deflate;

use std::fs::File;
use std::io;

pub mod graphicsstate;
use graphicsstate::{Color, Matrix};

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
    width: f32,
    height: f32,
}

impl Pdf {
    /// Create a new blank PDF document
    pub fn new() -> Self {
        let mut this = Pdf {
            buffer: Vec::new(),
            page_buffer: Vec::new(),
            // Object Catalog and Page Tree
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
        };
        // PDF magic header, should probably use a lower
        this.buffer.extend_from_slice(
            b"%PDF-1.7\n%\xB5\xED\xAE\xFB\n",
        );
        this
    }

    /// Move then pen, starting a new path
    fn move_to(&mut self, x: f32, y: f32) -> &mut Self {
        self.page_buffer.extend(
            format!("{:.2} {:.2} m ", x, y).bytes(),
        );
        self
    }

    /// Draw a line from the current location
    fn line_to(&mut self, x: f32, y: f32) -> &mut Self {
        self.page_buffer.extend(
            format!("{:.2} {:.2} l ", x, y).bytes(),
        );
        self
    }

    // Draw a cubic Bézier curve
    fn curve_to(&mut self, (x1, y1): (f32, f32), (x2, y2): (f32, f32), (x3, y3): (f32, f32))
        -> &mut Self {
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
    pub fn set_line_width(&mut self, width: f32) -> &mut Self {
        self.page_buffer.extend(format!("{:.2} w\n", width).bytes());
        self
    }

    /// Set the drawing color for the stroke operation,
    /// does not affect fill calls
    pub fn set_stroke_color(&mut self, color: Color) -> &mut Self {
        let norm = |color| color as f32 / 255.0;
        match color {
            Color::RGB { red, green, blue } => {
                self.page_buffer.extend(
                    format!(
                        "{:.2} {:.2} {:.2} SC\n",
                        norm(red),
                        norm(green),
                        norm(blue)
                    ).bytes(),
                )
            }
            Color::Gray { gray } => {
                self.page_buffer.extend(
                    format!("{:.2} G\n", norm(gray)).bytes(),
                )
            }
        };
        self
    }

    /// Apply a coordinate transformation to all subsequent drawing calls
    pub fn transform(&mut self, m: Matrix) -> &mut Self {
        self.page_buffer.extend(format!("{} cm\n", m).bytes());
        self
    }

    /// Draw a circle with the current drawing configuration,
    /// based on http://spencermortensen.com/articles/bezier-circle/
    pub fn draw_circle(&mut self, x: f32, y: f32, radius: f32) -> &mut Self {
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
    pub fn draw_line<I>(&mut self, mut points: I) -> &mut Self
    where
        I: Iterator<Item = (f32, f32)>,
    {
        if let Some((x, y)) = points.next() {
            self.move_to(x, y);
            for (x, y) in points {
                self.line_to(x, y);
            }
        }
        self.page_buffer.extend_from_slice(b"S\n");
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
        loop {
            let another = deflate::deflate_bytes_zlib(compressed.as_slice());
            if another.len() < compressed.len() {
                compressed = another;
                rounds += 1;
            } else {
                break;
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
        self.buffer.extend("\nendstream\nendobj\n".bytes());
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
        self.buffer.extend(
            format!("/MediaBox [0 0 {} {}]\n", self.width, self.height).bytes(),
        );
        self.buffer.extend(
            format!("/Contents {} 0 R", obj_id - 1).bytes(),
        );
        self.buffer.extend_from_slice(b">>\nendobj\n");
    }

    /// Move to a new page in the PDF document
    pub fn add_page(&mut self, width: f32, height: f32) -> &mut Self {
        // Compress and write out the previous page if it exists
        if !self.page_buffer.is_empty() {
            self.flush_page();
        }

        self.page_buffer.extend(
            "/DeviceRGB cs /DeviceRGB CS\n".bytes(),
        );
        self.width = width;
        self.height = height;
        self
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
