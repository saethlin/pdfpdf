//! Example program drawing some text on a page.
extern crate pdfpdf;

use pdfpdf::{BuiltinFont, Pdf};
use pdfpdf::graphicsstate::Color;

/// Create a `text.pdf` file, with a single page containg some
/// text lines positioned in various ways on some helper lines.
fn main() {
    Pdf::new()
        .set_title("Text example")
        .render_page(300.0, 400.0, |c| {
            c.set_stroke_color(Color::rgb(200, 200, 255));
            c.rectangle(10.0, 10.0, 280.0, 380.0);
            c.line(10.0, 300.0, 290.0, 300.0);
            c.line(150.0, 10.0, 150.0, 390.0);
            c.stroke();
            let helvetica = BuiltinFont::Helvetica;
            c.left_text(10.0, 380.0, helvetica, 12.0, "Top left");
            c.left_text(10.0, 10.0, helvetica, 12.0, "Bottom left");
            c.right_text(290.0, 380.0, helvetica, 12.0, "Top right");
            c.right_text(290.0, 10.0, helvetica, 12.0, "Bottom right");
            c.center_text(150.0, 330.0, BuiltinFont::Times_Bold, 18.0, "Centered");
            let times = c.get_font(BuiltinFont::Times_Roman);
            c.text(|t| {
                t.set_font(&times, 14.0);
                t.set_leading(18.0);
                t.pos(10.0, 300.0);
                t.show("Some lines of text in what might look like a");
                t.show_line("paragraph of three lines. Lorem ipsum dolor");
                t.show_line("sit amet. Blahonga. ");
                t.show_adjusted(&[("W", 130), ("AN", -40), ("D", 0)]);
                t.pos(0., -30.);
                t.show_adjusted(&(-19..21).map(|i| ("o", 16 * i)).collect::<Vec<_>>())
            });

            // In Swedish, we use the letters å, ä, and ö
            // in words like sloe liqueur.  That is why rust-pdf
            // uses /WinAnsiEncoding for text.
            let times_italic = BuiltinFont::Times_Italic;
            c.right_text(
                290.0,
                200.0,
                times_italic,
                14.0,
                "På svenska använder vi bokstäverna å, ä & ö",
            );
            c.right_text(
                290.0,
                182.0,
                times_italic,
                14.0,
                "i ord som slånbärslikör. Därför använder",
            );
            c.right_text(
                290.0,
                164.0,
                times_italic,
                14.0,
                "rust-pdf /WinAnsiEncoding för text.",
            );

            c.center_text(
                150.0,
                130.0,
                BuiltinFont::Symbol,
                14.0,
                "Hellas ΑΒΓΔαβγδ",
            );
            c.center_text(
                150.0,
                114.0,
                BuiltinFont::Symbol,
                14.0,
                "∀ μ < δ : ∃ σ ∈ Σ",
            );
        })
        .write_to("/tmp/text.pdf")
        .unwrap();
}
