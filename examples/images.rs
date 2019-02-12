extern crate pdfpdf;
extern crate png;

use pdfpdf::{Image, Pdf};
use std::fs::File;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let decoder = png::Decoder::new(File::open("examples/rustacean-flat-happy.png")?);
    let (info, mut reader) = decoder.read_info()?;
    let mut buf = vec![0; info.buffer_size()];
    reader.next_frame(&mut buf)?;

    let mut image_rgb = Vec::with_capacity(buf.len() * 3 / 4);
    for chunk in buf.chunks(4) {
        image_rgb.extend_from_slice(&chunk[..3]);
    }
    let image = Image::new(&image_rgb, info.width, info.height);

    Pdf::new_uncompressed()
        .add_page(info.width + 100, info.height + 100)
        .add_image(image, 50, 50)
        //.set_color(Color::rgb(0, 0, 248))
        //.draw_circle(0, 0, 1)
        .write_to("image_test.pdf")?;

    Ok(())
}
