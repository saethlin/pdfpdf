use fontsource::BuiltinFont;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, BufRead};

/// Relevant data that can be loaded from an AFM (Adobe Font Metrics) file.
/// A FontMetrics object is specific to a given encoding.
#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct FontMetrics {
    widths: BTreeMap<u8, u16>,
}

impl FontMetrics {
    /// Create a FontMetrics by reading an .afm file.
    pub fn parse(source: File) -> io::Result<FontMetrics> {
        let source = io::BufReader::new(source);
        let mut result = FontMetrics { widths: BTreeMap::new() };
        for line in source.lines() {
            let line = line.unwrap();
            let words: Vec<&str> = line.split_whitespace().collect();
            if words[0] == "C" && words[3] == "WX" {
                if let (Ok(c), Ok(w)) = (
                    words[1].parse::<u8>(),
                    words[4].parse::<u16>(),
                )
                {
                    result.widths.insert(c, w);
                }
            }
        }
        Ok(result)
    }

    /// Create a FontMetrics from a slice of (char, width) pairs.
    fn from_slice(data: &[(u8, u16)]) -> Self {
        let mut widths = BTreeMap::new();
        for &(c, w) in data {
            widths.insert(c, w);
        }
        FontMetrics { widths: widths }
    }

    /// Get the width of a specific character.
    /// The character is given in the encoding of the FontMetrics object.
    pub fn get_width(&self, char: u8) -> Option<u16> {
        match self.widths.get(&char) {
            Some(&w) => Some(w),
            None => None,
        }
    }
}

include!(concat!(env!("OUT_DIR"), "/metrics_data.rs"));
