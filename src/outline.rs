pub use encoding::WIN_ANSI_ENCODING;

/// An item in the document outline.
///
/// An OutlineItem associates a name (contained in an ordered tree)
/// with a location in the document.  The PDF standard supports
/// several ways to specify an exact location on a page, but this
/// implementation currently only supports linking to a specific page.
///
/// To actually create an OutlineItem in a meaningful way, please
/// use `Canvas::add_outline`.
#[derive(Clone)]
pub struct OutlineItem {
    title: String,
    page_id: Option<usize>,
}

impl OutlineItem {
    pub fn new(title: &str) -> OutlineItem {
        OutlineItem {
            title: title.to_string(),
            page_id: None,
        }
    }

    pub fn set_page(&mut self, page_id: usize) {
        self.page_id = Some(page_id)
    }

    pub fn write_dictionary(
        &self,
        output: &mut Vec<u8>,
        parent_id: usize,
        prev: Option<usize>,
        next: Option<usize>,
    ) {
        output.extend("<< /Title (".bytes());
        output.extend(
            format!("{:?}", &WIN_ANSI_ENCODING.encode_string(&self.title))
                .bytes(),
        );
        output.extend(")\n".bytes());
        output.extend(format!("/Parent {} 0 R\n", parent_id).bytes());
        if let Some(id) = prev {
            output.extend(format!("/Prev {} 0 R\n", id).bytes());
        }
        if let Some(id) = next {
            output.extend(format!("/Next {} 0 R\n", id).bytes());
        }
        if let Some(id) = self.page_id {
            output.extend(
                format!("/Dest [{} 0 R /XYZ null null null]\n", id).bytes(),
            );
        }
        output.extend(">>\n".bytes());
    }
}
