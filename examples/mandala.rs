///! Example program drawing mandalas on a page.
extern crate pdf_canvas;

use pdf_canvas::Pdf;
use pdf_canvas::graphicsstate::{Color, Matrix};
use std::env;
use std::f32::consts::PI;

/// Create a `mandala.pdf` file.
fn main() {
    // Open our pdf document.
    let mut document = Pdf::create("mandala.pdf").expect("Create PDF file");
    let mut args = env::args().skip(1);
    let n: u8 = args.next().map(|s| s.parse().expect("number")).unwrap_or(7);

    // Render a page with something resembling a mandala on it.
    document.render_page(600.0, 600.0, |c| {
        c.concat(Matrix::translate(300., 300.));
        c.set_stroke_color(Color::gray(0));
        let segment = 2. * PI / n as f32;
        for _i in 0..n {
            c.move_to(0., 33.5);
            c.line_to(0., 250.);
            let r = 99.;
            c.circle(0., r, r * 1.25 * segment);
            let d = 141.4;
            let rr = 36.;
            c.circle(0., d + rr, rr);
            c.stroke();
            c.concat(Matrix::rotate(segment));
        }
        c.concat(Matrix::rotate(segment / 2.));
        for _i in 0..n {
            let mut r0 = 58.66;
            let mut r = 0.7705 * r0 * segment;
            for _j in 0..(n + 1) / 3 {
                c.circle(0., r0, r);
                let r2 = 1.058 * r;
                r0 = r0 + r + r2;
                r = r2;
            }
            c.stroke();
            c.concat(Matrix::rotate(segment));
        }
    });
    document.finish().unwrap();
}
