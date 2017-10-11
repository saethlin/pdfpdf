extern crate pdfpdf;
use pdfpdf::{Font, Pdf};
use pdfpdf::Alignment::*;

fn main() {
    Pdf::new()
        .add_page(400, 400)
        .draw_line(vec![(0, 200), (400, 200)].into_iter())
        .draw_line(vec![(200, 0), (200, 400)].into_iter())
        .draw_text(0, 400, TopLeft, "Top\nLeft")
        .draw_text(400, 400, TopRight, "Top\nRight")
        .draw_text(200, 400, TopCenter, "Top\nCenter")
        .draw_text(0, 200, CenterLeft, "Center\nLeft")
        .draw_text(400, 200, CenterRight, "Center\nRight")
        .draw_text(200, 200, CenterCenter, "Center\nCenter")
        .draw_text(0, 0, BottomLeft, "Bottom\nLeft")
        .draw_text(400, 0, BottomRight, "Bottom\nRight")
        .draw_text(200, 0, BottomCenter, "Bottom\nCenter")
        .draw_text(0, 100, BottomLeft, "âàäçéèêëîïôùûü")
        // New page with the same, but now in TimesRoman
        .add_page(400, 400)
        .draw_line(vec![(0, 200), (400, 200)].into_iter())
        .draw_line(vec![(200, 0), (200, 400)].into_iter())
        .font(Font::Courier, 12)
        .draw_text(0, 400, TopLeft, "Top\nLeft")
        .draw_text(400, 400, TopRight, "Top\nRight")
        .draw_text(200, 400, TopCenter, "Top\nCenter")
        .draw_text(0, 200, CenterLeft, "Center\nLeft")
        .draw_text(400, 200, CenterRight, "Center\nRight")
        .draw_text(200, 200, CenterCenter, "Center\nCenter")
        .draw_text(0, 0, BottomLeft, "Bottom\nLeft")
        .draw_text(400, 0, BottomRight, "Bottom\nRight")
        .draw_text(200, 0, BottomCenter, "Bottom\nCenter")
        .draw_text(0, 100, BottomLeft, "âàäçéèêëîïôùûü")
        .write_to("text.pdf")
        .unwrap();
}
