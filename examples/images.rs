extern crate pdfpdf;
extern crate png;

use pdfpdf::Pdf;
use std::fs::File;

fn main() {
    let decoder = png::Decoder::new(File::open("examples/gantt.png").unwrap());
    let (info, mut reader) = decoder.read_info().unwrap();
    let mut buf = vec![0; info.buffer_size()];
    reader.next_frame(&mut buf).unwrap();

    let mut image_rgb = Vec::new();
    for (i, v) in buf.iter().enumerate() {
        if i % 4 != 0 {
            image_rgb.push(*v);
        }
    }

    Pdf::new_uncompressed()
        .add_page(700, 700)
        .add_image(info.width, info.height, image_rgb)
        .write_to("image_test.pdf")
        .unwrap();
}
