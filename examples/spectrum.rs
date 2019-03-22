use pdfpdf::{Color, Pdf, Size};

fn main() {
    let x: Vec<f64> = (0..400096).map(|n| n as f64 / 4096. * 600.).collect();
    let y: Vec<f64> = x
        .iter()
        .map(|x| (-(x - 300.0).powi(2) / 2400.0).exp() * 600.0)
        .collect();

    Pdf::new()
        .add_page(Size {
            width: 600,
            height: 600,
        })
        .set_color(Color::gray(100))
        .compression(pdfpdf::Compression::Off)
        .draw_line(x, y)
        .write_to("spectrum.pdf")
        .unwrap();
}
