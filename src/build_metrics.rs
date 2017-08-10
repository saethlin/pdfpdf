#[macro_use]
extern crate lazy_static;

use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Result, Write};
use std::path::Path;

#[allow(dead_code)]
mod encoding;
use encoding::{Encoding, SYMBOL_ENCODING, WIN_ANSI_ENCODING};

fn write_cond(f: &mut File, name: &str, encoding: &Encoding) -> Result<()> {
    try!(write!(f,
                "  static ref METRICS_{name}: FontMetrics = \
                 FontMetrics::from_slice(&[",
                name = name.to_uppercase()));
    let filename = format!("data/{}.afm", name.replace("_", "-"));
    println!("cargo:rerun-if-changed={}", filename);
    let afm_file = try!(File::open(filename));
    for lineresult in BufReader::new(afm_file).lines() {
        let line = try!(lineresult);
        let words: Vec<&str> = line.split_whitespace().collect();
        if words[0] == "C" && words[3] == "WX" && words[6] == "N" {
            if let (Some(c), Ok(w)) = (encoding.get_code(words[7]),
                                       words[4].parse::<u16>()) {
                try!(write!(f, "({}, {}), ", c, w));
            }
        }
    }
    try!(writeln!(f, "]);"));
    Ok(())
}

fn main() {
    let dst = Path::new(&env::var("OUT_DIR").unwrap()).join("metrics_data.rs");
    let mut f = &mut File::create(&dst).unwrap();
    let textfonts = ["Courier", "Courier_Bold",
                     "Courier_Oblique", "Courier_BoldOblique",
                     "Helvetica", "Helvetica_Bold",
                     "Helvetica_Oblique", "Helvetica_BoldOblique",
                     "Times_Roman", "Times_Bold",
                     "Times_Italic", "Times_BoldItalic"];
    writeln!(f,
             "pub fn get_builtin_metrics(font: &BuiltinFont) \
              -> &'static FontMetrics {{\n\
              match *font {{")
        .unwrap();
    for font in textfonts.iter().chain(["Symbol", "ZapfDingbats"].iter()) {
        writeln!(f,
                 "BuiltinFont::{} => &METRICS_{},",
                 font,
                 font.to_uppercase())
            .unwrap();
    }
    writeln!(f,
             "}}\n\
              }}\n\
              lazy_static! {{")
        .unwrap();
    for font in textfonts.iter() {
        write_cond(f, font, &WIN_ANSI_ENCODING).unwrap();
    }
    write_cond(f, "Symbol", &SYMBOL_ENCODING).unwrap();
    // FIXME There is a special encoding for ZapfDingbats
    write_cond(f, "ZapfDingbats", &WIN_ANSI_ENCODING).unwrap();
    writeln!(f, "}}").unwrap();
}
