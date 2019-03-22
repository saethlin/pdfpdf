use pdfpdf::{Alignment::*, Font, Pdf, Point, Size};

fn main() {
    Pdf::new()
        .add_page(Size {
            width: 400,
            height: 400,
        })
        //.draw_line([(0, 200), (400, 200)].into())
        //.draw_line([(200, 0), (200, 400)].into())
        .draw_text(Point { x: 0.0, y: 400 }, TopLeft, "Top\nLeft")
        .draw_text(Point { x: 400, y: 400 }, TopRight, "Top\nRight")
        .draw_text(Point { x: 200, y: 400 }, TopCenter, "Top\nCenter")
        .draw_text(Point { x: 0, y: 200 }, CenterLeft, "Center\nLeft")
        .draw_text(Point { x: 400, y: 200 }, CenterRight, "Center\nRight")
        .draw_text(Point { x: 200, y: 200 }, CenterCenter, "Center\nCenter")
        .draw_text(Point { x: 0, y: 0 }, BottomLeft, "Bottom\nLeft")
        .draw_text(Point { x: 400, y: 0 }, BottomRight, "Bottom\nRight")
        .draw_text(Point { x: 200, y: 0 }, BottomCenter, "Bottom\nCenter")
        .draw_text(
            Point { x: 0, y: 100 },
            BottomLeft,
            "âàäçéèêëîïôùûü",
        )
        // New page with the same, but now in TimesRoman
        .add_page(Size {
            width: 400,
            height: 400,
        })
        //.draw_line([(0, 200), (400, 200)].into())
        //.draw_line([(200, 0), (200, 400)].into())
        .font(Font::Courier, 12)
        .draw_text(Point { x: 0.0, y: 400 }, TopLeft, "Top\nLeft")
        .draw_text(Point { x: 400, y: 400 }, TopRight, "Top\nRight")
        .draw_text(Point { x: 200, y: 400 }, TopCenter, "Top\nCenter")
        .draw_text(Point { x: 0, y: 200 }, CenterLeft, "Center\nLeft")
        .draw_text(Point { x: 400, y: 200 }, CenterRight, "Center\nRight")
        .draw_text(Point { x: 200, y: 200 }, CenterCenter, "Center\nCenter")
        .draw_text(Point { x: 0, y: 0 }, BottomLeft, "Bottom\nLeft")
        .draw_text(Point { x: 400, y: 0 }, BottomRight, "Bottom\nRight")
        .draw_text(Point { x: 200, y: 0 }, BottomCenter, "Bottom\nCenter")
        .draw_text(
            Point { x: 0, y: 100 },
            BottomLeft,
            "âàäçéèêëîïôùûü",
        )
        .write_to("text.pdf")
        .unwrap();
}
