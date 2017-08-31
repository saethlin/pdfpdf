use std::char;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

fn main() {
    // We want a mapping from char to usize width
    // glyphlist.txt gives us a mapping from char to postscript name
    // .afm file gives us a mapping from postscript name to width

    let output_path = Path::new("src/fonts.rs");

    /*
    if output_path.is_file() {
        return;
    }
    */

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
    let mut name_to_width = Vec::new();
    let mut output = BufWriter::new(File::create(output_path).unwrap());
    write!(output, "#![allow(non_snake_case)]\n").unwrap();
    write!(output, "#![allow(unused_mut)]\n").unwrap();

    write!(output, "use std::collections::HashMap;\n\n").unwrap();
    write!(output, "lazy_static!{{\n").unwrap();
    write!(
        output,
        "    pub static ref GLYPH_WIDTHS: HashMap<Font, HashMap<char, f64>> = {{\n"
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
            name_to_width.push((name.to_owned(), width / 1000.0));
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

        write!(
            output,
            "        let mut {}_widths: HashMap<char, f64> = HashMap::new();\n",
            font_name
        ).unwrap();

        for &(chr, ref name) in &char_to_name {
            if let Some(&(_, width)) = name_to_width.iter().find(|&&(ref n, _)| *n == *name) {
                if chr == '\'' || chr == '\\' {
                    write!(
                        output,
                        "        {}_widths.insert('\\{}', {:.2});\n",
                        font_name,
                        chr,
                        width
                    ).unwrap();
                } else {
                    write!(
                        output,
                        "        {}_widths.insert('{}', {:.2});\n",
                        font_name,
                        chr,
                        width
                    ).unwrap();
                }
            }
        }
        write!(output, "\n").unwrap();
        name_to_width.clear();
    }

    // Write the hashmap from font enum to widths
    write!(output, "        let mut map = HashMap::new();\n").unwrap();
    for name in &font_names {
        write!(
            output,
            "        map.insert(Font::{}, {}_widths);\n",
            name,
            name
        ).unwrap();
    }
    write!(output, "        map\n").unwrap();
    write!(output, "    }};\n").unwrap();

    // Close the lazy_static invocation
    write!(output, "}}\n\n").unwrap();

    // Write the font enum
    write!(output, "#[derive(Hash, PartialEq, Eq)]\n").unwrap();
    write!(output, "pub enum Font {{\n").unwrap();
    for name in &font_names {
        write!(output, "    {},\n", name).unwrap();
    }
    write!(output, "}}\n").unwrap();
}
