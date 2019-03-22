//! Example program that draws a large number of dots on a page
use pdfpdf::{Pdf, Size};
use rand::distributions::Distribution;

/// Create a `dots.pdf` file, with a large number of dots normally
/// distributed in x and y about the center of the page
/// This is used as a benchmark in both time to build the PDF and
/// time to render it, as documents like this come up often in astronomy.
fn main() {
    let mut rng = rand::thread_rng();
    let dist = rand::distributions::Normal::new(200.0, 30.0);

    Pdf::new()
        .add_page(Size {
            width: 400,
            height: 400,
        })
        .transform(pdfpdf::Matrix::scale(2, 2) * pdfpdf::Matrix::translate(-200, -200))
        .draw_dots(
            &(0..100_000)
                .map(|_| dist.sample(&mut rng))
                .collect::<Vec<_>>(),
            &(0..100_000)
                .map(|_| dist.sample(&mut rng))
                .collect::<Vec<_>>(),
        )
        .write_to("dots.pdf")
        .unwrap();
}
