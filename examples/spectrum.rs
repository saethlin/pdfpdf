extern crate pdfpdf;
use pdfpdf::{Color, Pdf};

fn main() {
    let x: Vec<f64> = (0..4096).map(|n| n as f64 / 4096. * 600.).collect();
    let y: Vec<f64> = x
        .iter()
        .map(|x| (-(x - 300.0).powi(2) / 1200.0).exp() * 600.0)
        .collect();

    Pdf::new()
        .add_page(600.0, 600.0)
        .set_color(&Color::gray(100))
        .draw_line(x.iter().zip(y.iter()))
        .write_to("spectrum.pdf")
        .unwrap();
}
