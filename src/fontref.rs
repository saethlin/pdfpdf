use encoding::Encoding;
use fontmetrics::FontMetrics;
use std::fmt;
use std::sync::Arc;

/// A font ready to be used in a TextObject.
///
/// The way to get FontRef is to call
/// [Canvas::get_font](struct.Canvas.html#method.get_font) with a
/// [FontSource](trait.FontSource.html).
/// In PDF terms, a FontSource is everything needed to build a font
/// dictionary, while a FontRef is the name that can be used in a page
/// stream to use a font.
/// Calling Canvas::get_font will make sure the font dictionary is
/// created in the file, associate it with a name in the page
/// resources and return a FontRef representing that name.
///
/// The `serif` variable in
/// [the TextObject example](struct.TextObject.html#example) is a FontRef.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct FontRef {
    n: usize,
    encoding: Encoding,
    metrics: Arc<FontMetrics>,
}

// Should not be called by user code.
pub fn create_font_ref(n: usize,
                       encoding: Encoding,
                       metrics: Arc<FontMetrics>)
                       -> FontRef {
    FontRef {
        n: n,
        encoding: encoding,
        metrics: metrics,
    }
}

impl FontRef {
    /// Get the encoding used by the referenced font.
    pub fn get_encoding(&self) -> Encoding {
        self.encoding.clone()
    }

    /// Get the width of the given text in this font at given size.
    pub fn get_width(&self, size: f32, text: &str) -> f32 {
        size * self.get_width_raw(text) as f32 / 1000.0
    }

    /// Get the width of the given text in thousands of unit of text
    /// space.
    /// This unit is what is used in some places internally in pdf files
    /// and in some methods on a [TextObject](struct.TextObject.html).
    pub fn get_width_raw(&self, text: &str) -> u32 {
        let mut result = 0;
        for char in self.encoding.encode_string(text) {
            result += self.metrics.get_width(char).unwrap_or(100) as u32;
        }
        result
    }
}

impl fmt::Display for FontRef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "/F{}", self.n)
    }
}
