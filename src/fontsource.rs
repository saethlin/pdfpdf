use Pdf;
use encoding::{Encoding, SYMBOL_ENCODING, WIN_ANSI_ENCODING};
use fontmetrics::{FontMetrics, get_builtin_metrics};
use std::cmp::Eq;
use std::hash::Hash;
use std::ops::Add;

/// The "Base14" built-in fonts in PDF.
/// Underscores in these names are hyphens in the real names.
#[allow(non_camel_case_types, missing_docs)]
#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum BuiltinFont {
    Courier,
    Courier_Bold,
    Courier_Oblique,
    Courier_BoldOblique,
    Helvetica,
    Helvetica_Bold,
    Helvetica_Oblique,
    Helvetica_BoldOblique,
    Times_Roman,
    Times_Bold,
    Times_Italic,
    Times_BoldItalic,
    Symbol,
    ZapfDingbats,
}


/// This trait is implemented by any kind of font that the pdf library
/// supports.
///
/// Currently, only BuiltinFont implements this.
/// TODO Add implementation(s) for other fonts.
pub trait FontSource: PartialEq + Eq + Hash {
    /// Write the object(s) for this font to a pdf file.
    ///
    /// This is called automatically for each font used in a document.
    /// There should be no need to call this method from user code.
    fn write_object(&self, pdf: &mut Pdf) -> usize;

    /// Get the PDF name of this font.
    ///
    /// # Examples
    /// ```
    /// use pdfpdf::{BuiltinFont, FontSource};
    /// assert_eq!("Times-Roman", BuiltinFont::Times_Roman.pdf_name());
    /// ```
    fn pdf_name(&self) -> String;

    /// Get the encoding that this font uses.
    fn get_encoding(&self) -> Encoding;

    /// Get the width of a string in this font at given size.
    ///
    /// # Examples
    /// ```
    /// use pdfpdf::{BuiltinFont, FontSource};
    /// let proportional = BuiltinFont::Helvetica;
    /// assert_eq!(62.004, proportional.get_width(12.0, "Hello World"));
    /// let fixed = BuiltinFont::Courier;
    /// assert_eq!(60.0, fixed.get_width(10.0, "0123456789"));
    /// ```
    fn get_width(&self, size: f32, text: &str) -> f32;

    /// Get the width of a string in thousands of unit of text space.
    /// This unit is what is used in some places internally in pdf files.
    ///
    /// # Examples
    /// ```
    /// use pdfpdf::{BuiltinFont, FontSource};
    /// assert_eq!(5167, BuiltinFont::Helvetica.get_width_raw("Hello World"));
    /// assert_eq!(600, BuiltinFont::Courier.get_width_raw("A"));
    /// ```
    fn get_width_raw(&self, text: &str) -> u32;

    /// Get the font metrics for font.
    fn get_metrics(&self) -> FontMetrics;
}

impl FontSource for BuiltinFont {
    fn write_object(&self, pdf: &mut Pdf) -> usize {
        // Note: This is enough for a Base14 font, other fonts will
        // require a stream for the actual font, and probably another
        // object for metrics etc
        pdf.write_new_object(|font_object_id, pdf| {
            pdf.buffer.extend(
                format!(
                    "<< /Type /Font /Subtype /Type1 /BaseFont /{} \
                         /Encoding /{} >>\n",
                    self.pdf_name(),
                    self.get_encoding().get_name()
                ).bytes(),
            );
            font_object_id
        })
    }

    fn pdf_name(&self) -> String { format!("{:?}", self).replace("_", "-") }

    /// The encoding is WinAnsiEncoding for all builtin fonts except
    /// Symbol, for wich it is SymbolEncoding.
    /// TODO: ZapfDingbats should also have a special encoding.
    fn get_encoding(&self) -> Encoding {
        match self {
            &BuiltinFont::Symbol => SYMBOL_ENCODING.clone(),
            // &BuiltinFont::ZapfDingbats => ZAPFDINGBATS_ENCODING.clone(),
            _ => WIN_ANSI_ENCODING.clone(),
        }
    }

    fn get_width(&self, size: f32, text: &str) -> f32 {
        size * self.get_width_raw(text) as f32 / 1000.0
    }

    fn get_width_raw(&self, text: &str) -> u32 {
        let metrics = self.get_metrics();
        self.get_encoding()
            .encode_string(text)
            .iter()
            .map(|&ch| metrics.get_width(ch).unwrap_or(100) as u32)
            .fold(0, Add::add)
    }

    fn get_metrics(&self) -> FontMetrics { get_builtin_metrics(&self).clone() }
}
