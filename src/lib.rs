//! A library for creating pdf files.
//!
//! Currently, simple vector graphics and text set in the 14 built-in
//! fonts are supported.
//! The main entry point of the crate is the [struct Pdf](struct.Pdf.html),
//! representing a PDF file being written.

//! # Example
//!
//! ```
//! use pdfpdf::{Pdf, BuiltinFont, FontSource};
//! use pdfpdf::graphicsstate::Color;
//!
//! let mut document = Pdf::create("example.pdf")
//!     .expect("Create pdf file");
//! // The 14 builtin fonts are available
//! let font = BuiltinFont::Times_Roman;
//!
//! // Add a page to the document.  This page will be 180 by 240 pt large.
//! document.render_page(180.0, 240.0, |canvas| {
//!     // This closure defines the content of the page
//!     let hello = "Hello World!";
//!     let w = font.get_width(24.0, hello) + 8.0;
//!
//!     // Some simple graphics
//!     canvas.set_stroke_color(Color::rgb(0, 0, 248));
//!     canvas.rectangle(90.0 - w / 2.0, 194.0, w, 26.0);
//!     canvas.stroke();
//!
//!     // Some text
//!     canvas.center_text(90.0, 200.0, font, 24.0, hello)
//! });
//! // Write all pending content, including the trailer and index
//! document.finish().expect("Finish pdf document");
//! ```
//!
//! To use this library you need to add it as a dependency in your
//! `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! pdf-canvas = "*"
//! ```
//!
//! Some more working usage examples exists in [the examples directory]
//! (https://github.com/kaj/rust-pdf/tree/master/examples).
#![deny(missing_docs)]

#[macro_use]
extern crate lazy_static;
extern crate time;
extern crate deflate;

use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io;

mod fontsource;
pub use fontsource::{BuiltinFont, FontSource};

mod fontref;
pub use fontref::FontRef;

mod fontmetrics;
pub use fontmetrics::FontMetrics;

mod encoding;
pub use encoding::Encoding;

pub mod graphicsstate;

mod outline;
use outline::OutlineItem;

mod canvas;
pub use canvas::Canvas;

mod textobject;
pub use textobject::TextObject;

/// The top-level object for writing a PDF.
///
/// A PDF file is created with the `create` or `new` methods.
/// Some metadata can be stored with `set_foo` methods, and pages
/// are appended with the `render_page` method.
/// Don't forget to call `finish` when done, to write the document
/// trailer, without it the written file won't be a proper PDF.
pub struct Pdf {
    output: File,
    buffer: Vec<u8>,
    object_offsets: Vec<i64>,
    page_objects_ids: Vec<usize>,
    all_font_object_ids: HashMap<BuiltinFont, usize>,
    outline_items: Vec<OutlineItem>,
    document_info: HashMap<String, String>,
}

const ROOT_OBJECT_ID: usize = 1;
const PAGES_OBJECT_ID: usize = 2;

impl Pdf {
    /// Create a new PDF document as a new file with given filename.
    pub fn create(filename: &str) -> io::Result<Pdf> {
        let file = File::create(filename)?;
        Ok(Pdf::new(file))
    }

    /// Create a new PDF document, writing to `output`.
    pub fn new(output: File) -> Self {
        let mut this = Pdf {
            output: output,
            buffer: Vec::new(),
            // Object ID 0 is special in PDF.
            // We reserve IDs 1 and 2 for the catalog and page tree.
            object_offsets: vec![-1, -1, -1],
            page_objects_ids: vec![],
            all_font_object_ids: HashMap::new(),
            outline_items: Vec::new(),
            document_info: HashMap::new(),
        };
        // TODO Maybe use a lower version?  Possibly decide by features used?
        this.buffer.extend_from_slice(
            b"%PDF-1.7\n%\xB5\xED\xAE\xFB\n",
        );
        this
    }
    /// Set metadata: the document's title.
    pub fn set_title(&mut self, title: &str) {
        self.document_info.insert(
            "Title".to_string(),
            title.to_string(),
        );
    }
    /// Set metadata: the name of the person who created the document.
    pub fn set_author(&mut self, author: &str) {
        self.document_info.insert(
            "Author".to_string(),
            author.to_string(),
        );
    }
    /// Set metadata: the subject of the document.
    pub fn set_subject(&mut self, subject: &str) {
        self.document_info.insert(
            "Subject".to_string(),
            subject.to_string(),
        );
    }
    /// Set metadata: keywords associated with the document.
    pub fn set_keywords(&mut self, keywords: &str) {
        self.document_info.insert(
            "Subject".to_string(),
            keywords.to_string(),
        );
    }
    /// Set metadata: If the document was converted to PDF from another
    /// format, the name of the conforming product that created the original
    /// document from which it was converted.
    pub fn set_creator(&mut self, creator: &str) {
        self.document_info.insert(
            "Creator".to_string(),
            creator.to_string(),
        );
    }
    /// Set metadata: If the document was converted to PDF from another
    /// format, the name of the conforming product that converted it to PDF.
    pub fn set_producer(&mut self, producer: &str) {
        self.document_info.insert(
            "Producer".to_string(),
            producer.to_string(),
        );
    }

    /// Return the current read/write position in the output file.
    fn tell(&mut self) -> usize { self.buffer.len() }

    /// Create a new page in the PDF document.
    ///
    /// The page will be `width` x `height` points large, and the
    /// actual content of the page will be created by the function
    /// `render_contents` by applying drawing methods on the Canvas.
    pub fn render_page<F>(&mut self, width: f32, height: f32, render_contents: F)
    where
        F: FnOnce(&mut Canvas),
    {
        let (contents_object_id, content_length, fonts, outline_items) =
            self.write_new_object(move |contents_object_id, pdf| {
                use canvas::create_canvas;
                // TODO This is stupid, we don't need to use a dictionary since we know the size
                // Guess the ID of the next object. (We’ll assert it below.)
                pdf.buffer.extend(
                    format!(
                        "<< /Length {} 0 R /Filter /FlateDecode >>\nstream\n",
                        contents_object_id + 1
                    ).bytes(),
                );
                let mut compression_buffer = Vec::new();

                compression_buffer.extend("/DeviceRGB cs /DeviceRGB CS\n".bytes());
                let mut fonts = HashMap::new();
                let mut outline_items: Vec<OutlineItem> = Vec::new();
                render_contents(&mut create_canvas(
                    &mut compression_buffer,
                    &mut fonts,
                    &mut outline_items,
                ));

                let compressed = deflate::deflate_bytes_zlib(compression_buffer.as_slice());
                pdf.buffer.extend(compressed.iter());
                pdf.buffer.extend("\nendstream\n".bytes());
                (contents_object_id, compressed.len(), fonts, outline_items)
            });
        self.write_new_object(|length_object_id, pdf| {
            assert!(length_object_id == contents_object_id + 1);
            pdf.buffer.extend(format!("{}\n", content_length).bytes());
        });

        let mut font_oids = NamedRefs::new();
        for (src, r) in &fonts {
            if let Some(&object_id) = self.all_font_object_ids.get(&src) {
                font_oids.insert(r.clone(), object_id);
            } else {
                let object_id = src.write_object(self);
                font_oids.insert(r.clone(), object_id);
                self.all_font_object_ids.insert(*src, object_id);
            }
        }
        let page_oid = self.write_page_dict(contents_object_id, width, height, font_oids);
        // Take the outline_items from this page, mark them with the page ref,
        // and save them for the document outline.
        for i in &outline_items {
            let mut item = i.clone();
            item.set_page(page_oid);
            self.outline_items.push(item);
        }
        self.page_objects_ids.push(page_oid);
    }

    fn write_page_dict(&mut self, content_oid: usize, width: f32, height: f32, font_oids: NamedRefs)
        -> usize {
        self.write_new_object(|page_oid, pdf| {
            pdf.buffer.extend(
                format!(
                    "<< /Type /Page\n   \
                       /Parent {parent} 0 R\n   \
                       /Resources << /Font << {fonts}>> >>\n   \
                       /MediaBox [ 0 0 {width} {height} ]\n   \
                       /Contents {c_oid} 0 R\n\
                    >>\n",
                    parent = PAGES_OBJECT_ID,
                    fonts = font_oids,
                    width = width,
                    height = height,
                    c_oid = content_oid
                ).bytes(),
            );
            page_oid
        })
    }

    fn write_new_object<F, T>(&mut self, write_content: F) -> T
    where
        F: FnOnce(usize, &mut Pdf) -> T,
    {
        let id = self.object_offsets.len();
        let (result, offset) = self.write_object(id, |pdf| write_content(id, pdf));
        self.object_offsets.push(offset);
        result
    }

    fn write_object_with_id<F, T>(&mut self, id: usize, write_content: F) -> T
    where
        F: FnOnce(&mut Pdf) -> T,
    {
        assert!(self.object_offsets[id] == -1);
        let (result, offset) = self.write_object(id, write_content);
        self.object_offsets[id] = offset;
        result
    }

    fn write_object<F, T>(&mut self, id: usize, write_content: F) -> (T, i64)
    where
        F: FnOnce(&mut Pdf) -> T,
    {
        // `as i64` here would overflow for PDF files bigger than 2**63 bytes
        let offset = self.tell() as i64;
        self.buffer.extend(format!("{} 0 obj\n", id).bytes());
        let result = write_content(self);
        self.buffer.extend("endobj\n".bytes());
        (result, offset)
    }

    /// Write out the document trailer.
    /// The trailer consists of the pages object, the root object,
    /// the xref list, the trailer object and the startxref position.
    pub fn finish(mut self) -> io::Result<()> {
        self.write_object_with_id(PAGES_OBJECT_ID, |pdf| {
            pdf.buffer.extend(
                format!(
                    "<< /Type /Pages\n   \
                       /Count {c}\n   \
                       /Kids [ {pages}]\n\
                    >>\n",
                    c = pdf.page_objects_ids.len(),
                    pages = pdf.page_objects_ids
                        .iter()
                        .map(|id| format!("{} 0 R ", id))
                        .collect::<String>()
                ).bytes(),
            );
        });
        let document_info_id = if !self.document_info.is_empty() {
            let info = self.document_info.clone();
            self.write_new_object(|page_object_id, pdf| {
                write!(pdf.buffer, "<<").unwrap();
                for (key, value) in info {
                    write!(pdf.buffer, " /{} ({})\n", key, value).unwrap();
                }
                if let Ok(now) = time::strftime("%Y%m%d%H%M%S%z", &time::now()) {
                    write!(
                        pdf.buffer,
                        " /CreationDate (D:{now})\n \
                                  /ModDate (D:{now})",
                        now = now
                    ).unwrap();
                }
                write!(pdf.buffer, ">>\n").unwrap();
                Some(page_object_id)
            })
        } else {
            None
        };

        let outlines_id = self.write_outlines();

        self.write_object_with_id(ROOT_OBJECT_ID, |pdf| {
            write!(
                pdf.buffer,
                "<< /Type /Catalog\n   \
                            /Pages {} 0 R\n",
                PAGES_OBJECT_ID
            ).unwrap();
            if let Some(outlines_id) = outlines_id {
                write!(pdf.buffer, "/Outlines {} 0 R\n", outlines_id).unwrap();
            }
            write!(pdf.buffer, ">>\n").unwrap();
        });
        let startxref = self.tell();
        write!(
            self.buffer,
            "xref\n\
                     0 {}\n\
                     0000000000 65535 f \n",
            self.object_offsets.len()
        ).unwrap();
        // Object 0 (above) is special
        // Use [1..] to skip object 0 in self.object_offsets.
        for &offset in &self.object_offsets[1..] {
            assert!(offset >= 0);
            write!(self.buffer, "{:010} 00000 n \n", offset).unwrap();
        }
        write!(
            self.buffer,
            "trailer\n\
                     << /Size {size}\n   \
                        /Root {root} 0 R\n",
            size = self.object_offsets.len(),
            root = ROOT_OBJECT_ID
        ).unwrap();
        if let Some(id) = document_info_id {
            write!(self.buffer, "   /Info {} 0 R\n", id).unwrap();
        }

        write!(
            self.buffer,
            ">>\n\
                     startxref\n\
                     {}\n\
                     %%EOF\n",
            startxref
        ).unwrap();
        use std::io::Write;
        self.output.write_all(self.buffer.as_slice())
    }

    fn write_outlines(&mut self) -> Option<usize> {
        if self.outline_items.is_empty() {
            return None;
        }

        let parent_id = self.object_offsets.len();
        self.object_offsets.push(-1);
        let count = self.outline_items.len();
        let mut first_id = 0;
        let mut last_id = 0;
        let items = self.outline_items.clone();
        for (i, item) in items.iter().enumerate() {
            let (is_first, is_last) = (i == 0, i == count - 1);
            let id = self.write_new_object(|object_id, pdf| {
                item.write_dictionary(
                    &mut pdf.buffer,
                    parent_id,
                    if is_first { None } else { Some(object_id - 1) },
                    if is_last { None } else { Some(object_id + 1) },
                );
                object_id
            });
            if is_first {
                first_id = id;
            }
            if is_last {
                last_id = id;
            }
        }
        self.write_object_with_id(parent_id, |pdf| {
            pdf.buffer.extend(
                format!(
                    "<< /Type /Outlines\n   \
                    /First {first} 0 R\n   \
                    /Last {last} 0 R\n   \
                    /Count {count}\n\
                    >>\n",
                    last = last_id,
                    first = first_id,
                    count = count
                ).bytes(),
            );
        });
        Some(parent_id)
    }
}

struct NamedRefs {
    oids: HashMap<FontRef, usize>,
}

impl NamedRefs {
    fn new() -> Self { NamedRefs { oids: HashMap::new() } }
    fn insert(&mut self, name: FontRef, oid: usize) -> Option<usize> { self.oids.insert(name, oid) }
}


impl fmt::Display for NamedRefs {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (name, id) in self.oids.iter() {
            write!(f, "{} {} 0 R ", name, id)?;
        }
        Ok(())
    }
}
