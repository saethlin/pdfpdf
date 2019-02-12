// Rewrite a simple slideshow in Rust using pdfpdf
extern crate pdfpdf;
use pdfpdf::{Alignment, Color, Font, Pdf};

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
            pdf: Pdf::new_uncompressed(),
        }
    }

    pub fn add_title_slide(&mut self, text: &str) -> &mut Self {
        // init the new slide
        self.pdf
            .add_page(self.width, self.height)
            .set_color(self.background_color)
            .draw_rectangle_filled(0.0, 0.0, self.width, self.height)
            .font(Font::Helvetica, 100)
            .set_color(self.text_color)
            .draw_text(
                self.width / 2.0,
                self.height / 2.0,
                Alignment::CenterCenter,
                text,
            );
        self
    }

    pub fn add_text_slide(&mut self, text: &str) -> &mut Self {
        // init the new slide
        self.pdf
            .add_page(self.width, self.height)
            .set_color(self.background_color)
            .draw_rectangle_filled(0.0, 0.0, self.width, self.height)
            .font(Font::Helvetica, 60)
            .set_color(self.text_color)
            .draw_text(
                self.width / 2.0,
                self.height / 2.0,
                Alignment::CenterCenter,
                text,
            );
        self
    }

    pub fn write_to(&mut self, filename: &str) -> std::result::Result<(), std::io::Error> {
        self.pdf.write_to(filename)
    }
}
