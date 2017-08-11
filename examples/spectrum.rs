extern crate pdfpdf;
use pdfpdf::Pdf;
use pdfpdf::graphicsstate::Color;

fn main() {
    let x: Vec<f32> = (0..600).map(|n| n as f32).collect();
    let y: Vec<f32> = x.iter()
        .map(|x| (-(x - 300.0).powi(2) / 1200.0).exp() * 600.0)
        .collect();

    Pdf::new()
        .render_page(600.0, 600.0, |c| {
            c.set_stroke_color(Color::gray(100));
            c.move_to(x[0], y[0]);
            for (x, y) in x.iter().zip(y.iter()) {
                c.line_to(*x, *y);
            }
            c.stroke();
        })
        .write_to("/tmp/spectrum.pdf")
        .unwrap();
}
