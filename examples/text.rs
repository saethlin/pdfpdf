extern crate pdfpdf;
use pdfpdf::Alignment::*;
use pdfpdf::{Font, Pdf};

fn main() {
    Pdf::new()
        .add_page(400, 400)
        .draw_line([0, 400].iter().zip([200, 200].iter()))
        .draw_line([200, 200].iter().zip([0, 400].iter()))
        .draw_text(0.0, 400, TopLeft, "Top\nLeft")
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
        .draw_line([0, 400].iter().zip([200, 200].iter()))
        .draw_line([200, 200].iter().zip([0, 400].iter()))
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
