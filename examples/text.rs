extern crate pdfpdf;
use pdfpdf::{Font, Pdf};
use pdfpdf::Alignment::*;

fn main() {
    Pdf::new()
        .add_page(400, 400)
        .font(Font::HelveticaBold, 20)
        .draw_text(200, 300, BottomCenter, "Centered")
        .font(Font::Helvetica, 12)
        .draw_text(0, 0, BottomLeft, "Bottom left")
        .draw_text(400, 0, BottomRight, "Bottom right")
        .draw_text(0, 400, TopLeft, "Top left")
        .draw_text(400, 400, TopRight, "Top right")
        .draw_text(
            0,
            200,
            BottomLeft,
            "På svenska använder vi bokstäverna å, ä & ö i ord som slånbärslikör.",
        )
        .write_to("text.pdf")
        .unwrap();
}
