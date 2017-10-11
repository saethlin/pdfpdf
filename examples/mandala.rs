///! Example program drawing mandalas on a page.
extern crate pdfpdf;

use pdfpdf::{Color, Matrix, Pdf};
use std::env;
use std::f32::consts::PI;

fn main() {
    let mut args = env::args().skip(1);
    let n: usize = args.next().map(|s| s.parse().expect("number")).unwrap_or(7);

    let angle = 2.0 * PI / n as f32;
    let r = 99.0;
    let d = 141.4;
    let rr = 36.0;

    let mut document = Pdf::new();
    document
        .add_page(600.0, 600.0)
        .transform(Matrix::translate(300.0, 300.0))
        .set_color(&Color::gray(0));

    for _ in 0..n {
        document
            .draw_line(vec![(0.0, 33.5), (0.0, 250.0)].into_iter())
            .draw_circle(0.0, r, r * 1.25 * angle)
            .draw_circle(0.0, d + rr, rr)
            .transform(Matrix::rotate(angle));
    }
    document.transform(Matrix::rotate(angle / 2.0));
    for _ in 0..n {
        let mut r0 = 58.66;
        let mut r = 0.7705 * r0 * angle;
        for _ in 0..(n + 1) / 3 {
            document.draw_circle(0., r0, r);
            let r2 = 1.058 * r;
            r0 += r + r2;
            r = r2;
        }
        document.transform(Matrix::rotate(angle));
    }

    document.write_to("mandala.pdf").unwrap();
}
