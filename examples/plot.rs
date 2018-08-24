extern crate pdfpdf;
use pdfpdf::{Color, Pdf};
use std::io;

pub struct Plot {
    pdf: Pdf,
    width: f64,
    height: f64,
}

impl Plot {
    pub fn new() -> Self {
        Self {
            pdf: Pdf::new_uncompressed(),
            width: 500.0,
            height: 500.0,
        }
    }

    pub fn width(&mut self, width: f64) -> &mut Self {
        self.width = width;
        self
    }

    pub fn height(&mut self, height: f64) -> &mut Self {
        self.height = height;
        self
    }

    pub fn plot<'a, I, N: 'a>(&mut self, points: I) -> &mut Self
    where
        I: Iterator<Item = (&'a N, &'a N)>,
        N: Into<f64> + Copy,
    {
        let border = 0.05f64;
        self.pdf
            .add_page(self.width, self.height)
            .set_color(&Color::gray(0))
            .draw_line(
                [
                    border * self.width,
                    (1.0 - border) * self.width,
                    (1.0 - border) * self.width,
                    border * self.width,
                    border * self.width,
                ].iter()
                    .zip(
                        [
                            border * self.height,
                            border * self.height,
                            (1.0 - border) * self.height,
                            (1.0 - border) * self.height,
                            border * self.height,
                        ].iter(),
                    ),
            )
            .set_color(&Color::rgb(100, 100, 255))
            .draw_line(points);
        self
    }

    pub fn write_to(&mut self, filename: &str) -> io::Result<()> {
        self.pdf.write_to(filename)
    }
}

fn main() {
    let x: Vec<f32> = (0..4096).map(|n| n as f32 / 4096. * 600.).collect();
    let y: Vec<f32> = x.iter()
        .map(|x| (-(x - 300.0).powi(2) / 1200.0).exp() * 600.0)
        .collect();

    Plot::new()
        .plot(x.iter().zip(y.iter()))
        .write_to("plot.pdf")
        .unwrap();
}
