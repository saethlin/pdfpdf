//! Example program drawing circles on a page.

use pdfpdf::{Color, Pdf, Point, Size};
use std::f64::consts::PI;

/// Create a `circles.pdf` file, with a single page containg a circle
/// stroked in black, overwritten with a circle in a finer yellow
/// stroke.
/// The black circle is drawn using the `Pdf::draw_circle` method,
/// which approximates a circle with four bezier curves.
/// The yellow circle is drawn as a 200-sided polygon.
fn main() {
    let (x, y) = (200.0, 200.0);
    let r = 190.0;
    let sides = 200;
    let angles = (0..=sides).map(|n| 2. * PI * n as f64 / sides as f64);

    Pdf::new()
        .add_page(Size {
            width: 400.0,
            height: 400.0,
        })
        .set_color(Color::gray(0))
        .set_line_width(2.0)
        .draw_circle(Point { x, y }, r)
        .set_color(Color {
            red: 255,
            green: 230,
            blue: 150,
        })
        .set_line_width(1.0)
        .draw_line(
            angles.clone().map(|phi| x + r * phi.cos()),
            angles.map(|phi| y + r * phi.sin()),
        )
        .write_to("circles.pdf")
        .unwrap();
}
