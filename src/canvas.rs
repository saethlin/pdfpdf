use fontref::FontRef;
use fontsource::{BuiltinFont, FontSource};
use graphicsstate::*;
use outline::OutlineItem;
use std::collections::HashMap;
use std::sync::Arc;
use textobject::TextObject;

/// An visual area where content can be drawn (a page).
///
/// Provides methods for defining and stroking or filling paths, as
/// well as placing text objects.
///
/// TODO Everything here that takes a `BuiltinFont` should take any
/// `FontSource` instead.
pub struct Canvas<'a> {
    output: &'a mut Vec<u8>,
    fonts: &'a mut HashMap<BuiltinFont, FontRef>,
    outline_items: &'a mut Vec<OutlineItem>,
}

// Should not be called by user code.
pub fn create_canvas<'a>(
    output: &'a mut Vec<u8>, fonts: &'a mut HashMap<BuiltinFont, FontRef>,
    outline_items: &'a mut Vec<OutlineItem>
) -> Canvas<'a> {
    Canvas {
        output: output,
        fonts: fonts,
        outline_items: outline_items,
    }
}

impl<'a> Canvas<'a> {
    /// Append a closed rectangle with a corner at (x, y) and
    /// extending width × height to the to the current path.
    pub fn rectangle(&mut self, x: f32, y: f32, width: f32, height: f32) {
        self.output.extend(
            format!("{:.2} {:.2} {:.2} {:.2} re\n", x, y, width, height).bytes(),
        );
    }
    /// Set the line join style in the graphics state.
    pub fn set_line_join_style(&mut self, style: JoinStyle) {
        self.output.extend(
            format!(
                "{} j\n",
                match style {
                    JoinStyle::Miter => 0,
                    JoinStyle::Round => 1,
                    JoinStyle::Bevel => 2,
                }
            ).bytes(),
        );
    }
    /// Set the line join style in the graphics state.
    pub fn set_line_cap_style(&mut self, style: CapStyle) {
        self.output.extend(
            format!(
                "{} J\n",
                match style {
                    CapStyle::Butt => 0,
                    CapStyle::Round => 1,
                    CapStyle::ProjectingSquare => 2,
                }
            ).bytes(),
        );
    }
    /// Set the line width in the graphics state.
    pub fn set_line_width(&mut self, w: f32) {
        self.output.extend(format!("{} w\n", w).bytes());
    }
    /// Set color for stroking operations.
    pub fn set_stroke_color(&mut self, color: Color) {
        let norm = |c| c as f32 / 255.0;
        match color {
            Color::RGB { red, green, blue } => {
                self.output.extend(
                    format!("{} {} {} SC\n", norm(red), norm(green), norm(blue)).bytes(),
                );
            }
            Color::Gray { gray } => {
                self.output.extend(format!("{} G\n", norm(gray)).bytes());
            }
        }
    }
    /// Set color for non-stroking operations.
    pub fn set_fill_color(&mut self, color: Color) {
        let norm = |c| c as f32 / 255.0;
        match color {
            Color::RGB { red, green, blue } => {
                self.output.extend(
                    format!("{} {} {} sc\n", norm(red), norm(green), norm(blue)).bytes(),
                );
            }
            Color::Gray { gray } => {
                self.output.extend(format!("{} g\n", norm(gray)).bytes());
            }
        }
    }

    /// Modify the current transformation matrix for coordinates by
    /// concatenating the specified matrix.
    pub fn concat(&mut self, m: Matrix) {
        self.output.extend(format!("{} cm\n", m).bytes());
    }

    /// Append a straight line from (x1, y1) to (x2, y2) to the current path.
    pub fn line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32) {
        self.move_to(x1, y1);
        self.line_to(x2, y2);
    }
    /// Begin a new subpath at the point (x, y).
    pub fn move_to(&mut self, x: f32, y: f32) {
        self.output.extend(format!("{:.2} {:.2} m ", x, y).bytes());
    }
    /// Add a straight line from the current point to (x, y) to the
    /// current path.
    pub fn line_to(&mut self, x: f32, y: f32) {
        self.output.extend(format!("{:.2} {:.2} l ", x, y).bytes());
    }
    /// Add an Bézier curve from the current point to (x3, y3) with
    /// (x1, y1) and (x2, y2) as Bézier control points.
    pub fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x3: f32, y3: f32) {
        self.output.extend(
            format!(
                "{:.2} {:.2} {:.2} {:.2} {:.2} {:.2} c\n",
                x1,
                y1,
                x2,
                y2,
                x3,
                y3
            ).bytes(),
        );
    }
    /// Add a circle approximated by four cubic Bézier curves to the
    /// current path.  Based on
    /// http://spencermortensen.com/articles/bezier-circle/
    pub fn circle(&mut self, x: f32, y: f32, r: f32) {
        let t = y - r;
        let b = y + r;
        let left = x - r;
        let right = x + r;
        let c = 0.551915024494;
        let leftp = x - (r * c);
        let rightp = x + (r * c);
        let tp = y - (r * c);
        let bp = y + (r * c);
        self.move_to(x, t);
        self.curve_to(leftp, t, left, tp, left, y);
        self.curve_to(left, bp, leftp, b, x, b);
        self.curve_to(rightp, b, right, bp, right, y);
        self.curve_to(right, tp, rightp, t, x, t);
    }
    /// Stroke the current path.
    pub fn stroke(&mut self) {
        self.output.extend("S\n".bytes());
    }
    /// Close and stroke the current path.
    pub fn close_and_stroke(&mut self) {
        self.output.extend("s\n".bytes());
    }
    /// Fill the current path.
    pub fn fill(&mut self) {
        self.output.extend("f\n".bytes());
    }
    /// Get a FontRef for a specific font.
    pub fn get_font(&mut self, font: BuiltinFont) -> FontRef {
        use fontref::create_font_ref;
        let next_n = self.fonts.len();
        self.fonts
            .entry(font)
            .or_insert_with(|| {
                create_font_ref(next_n, font.get_encoding(), Arc::new(font.get_metrics()))
            })
            .clone()
    }

    /// Create a text object.
    ///
    /// The contents of the text object is defined by the function
    /// render_text, by applying methods to the TextObject it gets as
    /// an argument.
    /// On success, return the value returned by render_text.
    pub fn text<F, T>(&mut self, render_text: F) -> T
    where
        F: FnOnce(&mut TextObject) -> T,
    {
        use textobject::create_text_object;
        self.output.extend("BT\n".bytes());
        let result = render_text(&mut create_text_object(self.output));
        self.output.extend("ET\n".bytes());
        result
    }
    /// Utility method for placing a string of text.
    pub fn left_text(&mut self, x: f32, y: f32, font: BuiltinFont, size: f32, text: &str) {
        let font = self.get_font(font);
        self.text(|t| {
            t.set_font(&font, size);
            t.pos(x, y);
            t.show(text);
        })
    }
    /// Utility method for placing a string of text.
    pub fn right_text(&mut self, x: f32, y: f32, font: BuiltinFont, size: f32, text: &str) {
        let font = self.get_font(font);
        self.text(|t| {
            let text_width = font.get_width(size, text);
            t.set_font(&font, size);
            t.pos(x - text_width, y);
            t.show(text);
        })
    }
    /// Utility method for placing a string of text.
    pub fn center_text(&mut self, x: f32, y: f32, font: BuiltinFont, size: f32, text: &str) {
        let font = self.get_font(font);
        self.text(|t| {
            let text_width = font.get_width(size, text);
            t.set_font(&font, size);
            t.pos(x - text_width / 2.0, y);
            t.show(text);
        })
    }

    /// Add an item for this page in the document outline.
    ///
    /// An outline item associates a name (contained in an ordered
    /// tree) with a location in the document.  The PDF standard
    /// supports several ways to specify an exact location on a page,
    /// but this implementation currently only supports linking to a
    /// specific page (the page that this Canvas is for).
    pub fn add_outline(&mut self, title: &str) {
        self.outline_items.push(OutlineItem::new(title));
    }

    /// Save the current graphics state.
    /// The caller is responsible for restoring it later.
    pub fn gsave(&mut self) {
        self.output.extend("q\n".bytes());
    }
    /// Restor the current graphics state.
    /// The caller is responsible for having saved it earlier.
    pub fn grestore(&mut self) {
        self.output.extend("Q\n".bytes());
    }
}
