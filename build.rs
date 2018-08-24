use std::char;
use std::collections::HashMap;
use std::fmt::Write;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

fn main() {
    // We want a mapping from char to usize width
    // glyphlist.txt gives us a mapping from char to postscript name
    // .afm file gives us a mapping from postscript name to width

    let output_path = Path::new("src/fonts.rs");

    let mut char_to_name = Vec::new();
    for line in BufReader::new(File::open("data/glyphlist.txt").unwrap())
        .lines()
        .filter_map(|e| e.ok())
        .filter(|l| !l.starts_with('#'))
    {
        let fields: Vec<&str> = line.split(';').collect();
        let numbers: Vec<char> = fields[1]
            .split_whitespace()
            .map(|c| u32::from_str_radix(c, 16).unwrap())
            .map(|c| char::from_u32(c).unwrap())
            .collect();
        for num in numbers {
            char_to_name.push((num, fields[0].to_owned()));
        }
    }

    let mut font_names = Vec::new();
    let mut name_to_width = HashMap::new();
    let mut output = String::new();
    write!(output, "#![allow(non_snake_case)]\n").unwrap();
    write!(output, "#![allow(missing_docs)]\n").unwrap();

    write!(
        output,
        "pub fn glyph_width(font: &Font, c: char) -> f64 {{\n    match font {{\n"
    ).unwrap();

    for entry in std::fs::read_dir(Path::new("data/Core14_AFMs"))
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().unwrap().is_file())
        .filter(|e| e.file_name().to_str().unwrap().ends_with(".afm"))
    {
        for line in BufReader::new(File::open(entry.path()).unwrap())
            .lines()
            .filter_map(|e| e.ok())
            .skip_while(|line| !line.starts_with("StartCharMetrics"))
            .skip(1)
            .take_while(|line| !line.starts_with("EndCharMetrics"))
        {
            let fields: Vec<&str> = line.split(' ').collect();
            let width: f64 = fields[4].parse().unwrap();
            let name = fields[7];
            name_to_width.insert(name.to_owned(), width / 1000.0);
        }

        let font_name = entry
            .file_name()
            .to_str()
            .unwrap()
            .split('.')
            .next()
            .unwrap()
            .replace('-', "")
            .to_owned();
        font_names.push(font_name.clone());

        write!(output, "        &Font::{} => match c {{\n", font_name).unwrap();

        for &(chr, ref name) in &char_to_name {
            if let Some(&width) = name_to_width.get(name) {
                if chr == '\'' || chr == '\\' {
                    write!(output, "            '\\{}' => {:.2},\n", chr, width).unwrap();
                } else {
                    write!(output, "            '{}' => {:.2},\n", chr, width).unwrap();
                }
            }
        }
        write!(output, "            _ => 0.0,\n").unwrap();
        write!(output, "        }}\n").unwrap();
        name_to_width.clear();
    }
    write!(output, "    }}\n").unwrap();

    // Close the lazy_static invocation
    write!(output, "}}\n\n").unwrap();

    // Write the font enum
    write!(output, "#[derive(Clone, Debug, Eq, Hash, PartialEq)]\n").unwrap();
    write!(output, "pub enum Font {{\n").unwrap();
    for name in &font_names {
        write!(output, "    {},\n", name).unwrap();
    }
    write!(output, "}}\n").unwrap();

    // Write to output file only if we need to
    let mut current_contents = String::new();
    if let Ok(mut f) = File::open(output_path) {
        use std::io::Read;
        f.read_to_string(&mut current_contents).unwrap();
    }
    if current_contents != output {
        use std::io::Write;
        let mut output_file = File::create(output_path).unwrap();
        write!(output_file, "{}", output).unwrap();
    }
}
