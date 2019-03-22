///! Example program drawing mandalas on a page.
use pdfpdf::{Color, Matrix, Pdf, Point, Size};
use std::env;
use std::f64::consts::PI;

fn main() {
    let mut args = env::args().skip(1);
    let n: usize = args.next().map(|s| s.parse().expect("number")).unwrap_or(7);

    let angle = 2.0 * PI / n as f64;
    let r = 99.0;
    let d = 141.4;
    let rr = 36.0;

    let mut document = Pdf::new();
    document
        .add_page(Size {
            width: 600,
            height: 600,
        })
        .transform(Matrix::translate(300.0, 300.0))
        .set_color(Color::gray(0));

    for _ in 0..n {
        document
            .move_to(Point { x: 0.0, y: 33.5 })
            .line_to(Point { x: 0.0, y: 250.0 })
            .draw_circle(Point { x: 0.0, y: r }, r * 1.25 * angle)
            .draw_circle(Point { x: 0.0, y: d + rr }, rr)
            .transform(Matrix::rotate(angle));
    }
    document.transform(Matrix::rotate(angle / 2.0));
    for _ in 0..n {
        let mut r0 = 58.66;
        let mut r = 0.7705 * r0 * angle;
        for _ in 0..(n + 1) / 3 {
            document.draw_circle(Point { x: 0., y: r0 }, r);
            let r2 = 1.058 * r;
            r0 += r + r2;
            r = r2;
        }
        document.transform(Matrix::rotate(angle));
    }

    document.write_to("mandala.pdf").unwrap();
}
