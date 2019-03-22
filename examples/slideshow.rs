//! Rewrite a simple slideshow in Rust using pdfpdf
use pdfpdf::{Alignment, Color, Font, Pdf, Point, Size};

fn main() {
    Slideshow::new(1280, 1024, Color::gray(0), Color::gray(255))
        .add_title_slide("Lessons from LATHER")
        .add_text_slide("The Activity Problem\nOR\nRemove the spots")
        .add_text_slide("1. Find/make a good model\n2. Run it. A lot.\n3. Listen at group meetings")
        .add_text_slide("Easy to use\nWe're going to write a lot of scripts")
        .add_text_slide("SOAP: 2.4 s\nLATHER: 0.006 s")
        .add_text_slide("All I Really Need to Know I Learned in\nKindergarten")
        .add_text_slide("All I Really Need to Know I Learned in\nMathematical Physics")
        .write_to("lessons_from_lather.pdf")
        .expect("Couldn't save slideshow");
}

struct Slideshow {
    width: f64,
    height: f64,
    background_color: Color,
    text_color: Color,
    pdf: Pdf,
}

impl Slideshow {
    pub fn new<N1: Into<f64>, N2: Into<f64>>(
        width: N1,
        height: N2,
        background_color: Color,
        text_color: Color,
    ) -> Self {
        Slideshow {
            width: width.into(),
            height: height.into(),
            background_color,
            text_color,
            pdf: {
                let mut pdf = Pdf::new();
                pdf.compression(pdfpdf::Compression::Off);
                pdf
            },
        }
    }

    pub fn add_title_slide(&mut self, text: &str) -> &mut Self {
        // init the new slide
        self.pdf
            .add_page(Size {
                width: self.width,
                height: self.height,
            })
            .set_color(self.background_color)
            .draw_rectangle_filled(
                Point { x: 0, y: 0 },
                Size {
                    width: self.width,
                    height: self.height,
                },
            )
            .font(Font::Helvetica, 100)
            .set_color(self.text_color)
            .draw_text(
                Point {
                    x: self.width / 2.0,
                    y: self.height / 2.0,
                },
                Alignment::CenterCenter,
                text,
            );
        self
    }

    pub fn add_text_slide(&mut self, text: &str) -> &mut Self {
        // init the new slide
        self.pdf
            .add_page(Size {
                width: self.width,
                height: self.height,
            })
            .set_color(self.background_color)
            .draw_rectangle_filled(
                Point { x: 0, y: 0 },
                Size {
                    width: self.width,
                    height: self.height,
                },
            )
            .font(Font::Helvetica, 60)
            .set_color(self.text_color)
            .draw_text(
                Point {
                    x: self.width / 2.0,
                    y: self.height / 2.0,
                },
                Alignment::CenterCenter,
                text,
            );
        self
    }

    pub fn write_to(&mut self, filename: &str) -> std::result::Result<(), std::io::Error> {
        self.pdf.write_to(filename)
    }
}
