//! Example program drawing circles on a page.
extern crate pdfpdf;

use pdfpdf::{Color, Pdf};
use std::f32::consts::PI;

/// Create a `circles.pdf` file, with a single page containg a circle
/// stroked in black, overwritten with a circle in a finer yellow
/// stroke.
/// The black circle is drawn using the `Canvas.circle` method,
/// which approximates a circle with four bezier curves.
/// The yellow circle is drawn as a 200-sided polygon.
fn main() {
    let (x, y) = (200.0, 200.0);
    let r = 190.0;
    let sides = 200;
    let angles = (0..sides).map(|n| 2. * PI * n as f32 / sides as f32);

    Pdf::new()
        .add_page(400.0, 400.0)
        .set_stroke_color(&Color::rgb(0, 0, 0))
        .set_line_width(2.0)
        .draw_circle(x, y, r)
        .set_stroke_color(&Color::rgb(255, 230, 150))
        .set_line_width(1.0)
        .draw_line(angles.map(|phi| (x + r * phi.cos(), y + r * phi.sin())))
        .write_to("circles.pdf")
        .unwrap();
}
