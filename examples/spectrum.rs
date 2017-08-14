extern crate pdfpdf;
use pdfpdf::Pdf;
use pdfpdf::graphicsstate::Color;

fn main() {
    let x: Vec<f32> = (0..600).map(|n| n as f32).collect();
    let y: Vec<f32> = x.iter()
        .map(|x| (-(x - 300.0).powi(2) / 1200.0).exp() * 600.0)
        .collect();

    Pdf::new()
        .add_page(600.0, 600.0)
        .set_stroke_color(Color::gray(100))
        .draw_line(x.into_iter().zip(y.into_iter()))
        .write_to("spectrum.pdf")
        .unwrap();
}
