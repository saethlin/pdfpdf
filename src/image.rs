/// A wrapper around a buffer and dimensions to make drawing images more ergonomic
#[derive(Clone, Copy)]
pub struct Image<'a> {
    pub(crate) buf: &'a [u8],
    pub(crate) width: u64,
    pub(crate) height: u64,
}

impl<'a> Image<'a> {
    /// Create an Image from some bytes, panics if buffer length is not a multiple of 3 or if the
    /// product of the width and height is not the buffer length
    pub fn new<N1, N2>(buf: &'a [u8], width: N1, height: N2) -> Image<'a>
    where
        u64: From<N1>,
        u64: From<N2>,
    {
        let width = u64::from(width);
        let height = u64::from(height);
        assert_eq!(buf.len() % 3, 0);
        assert_eq!(width * height * 3, buf.len() as u64);
        Image { buf, width, height }
    }
}
