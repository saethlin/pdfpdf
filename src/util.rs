// tt muncher
macro_rules! ryu {
    ($buffer:expr, $($tail:tt)*) => {
        {
            let mut __ryu_buffer = ryu::Buffer::new();
            ryu_intern!($buffer, &mut __ryu_buffer, $($tail)*);
        }
    };
}

macro_rules! ryu_intern {
    ($out:expr, $ryubuf:expr, $item:expr) => {
        $item.ryu_format(&mut $out, $ryubuf);
        $out.push(b'\n');
    };
    ($out:expr, $ryubuf:expr, $item:expr, $($tail:tt)*) => {
        $item.ryu_format(&mut $out, $ryubuf);
        $out.push(b' ');
        ryu_intern!($out, $ryubuf, $($tail)*);
    };
}

pub trait Formattable {
    fn ryu_format(self, out: &mut Vec<u8>, ryubuf: &mut ryu::Buffer);
}

impl Formattable for f64 {
    #[inline]
    #[allow(clippy::float_cmp)]
    fn ryu_format(mut self, out: &mut Vec<u8>, ryubuf: &mut ryu::Buffer) {
        if self < 0.0 {
            self *= -1.0;
            out.push(b'-');
        }
        // These are the majority of calls for intensive matplotlib-style code
        // because we spend a lot of time printing transofmation matrices
        if self == 1.0 {
            out.push(b'1');
            return;
        } else if self == 0.0 {
            out.push(b'0');
            return;
        }
        // Use ryu for numbers in the range where it doesn't use scientific notation
        if 1e-5 < self && self < 1e16 {
            let digits = &ryubuf.format(self).as_bytes();
            let dot_index = digits.iter().position(|b| *b == b'.').unwrap_or(0);
            // Try to trim if the number contains a lot of decimal precision
            if dot_index < 14 {
                // TODO: This truncation should be a smart rounding of some sort
                let digits = &digits[..(digits.len().min(dot_index + 11))];
                // We can try to trim away some of the zeroes on the right
                let num_nonzero = digits
                    .iter()
                    .rev()
                    .skip_while(|b| **b == b'0')
                    .skip_while(|b| **b == b'.')
                    .count();
                out.extend_from_slice(&digits[..num_nonzero]);
            } else {
                out.extend_from_slice(digits);
            }
        } else {
            out.extend_from_slice(format!("{}", self).as_bytes());
        }
    }
}

impl Formattable for &str {
    #[inline]
    fn ryu_format(self, out: &mut Vec<u8>, _: &mut ryu::Buffer) {
        out.extend_from_slice(self.as_bytes())
    }
}

#[allow(missing_docs)]
#[derive(Clone, Copy, Debug)]
pub struct Point<X, Y> {
    pub x: X,
    pub y: Y,
}

impl<X, Y> Point<X, Y>
where
    X: Into<f64>,
    Y: Into<f64>,
{
    pub(crate) fn into_f64(self) -> Point<f64, f64> {
        Point {
            x: self.x.into(),
            y: self.y.into(),
        }
    }
}

#[allow(missing_docs)]
#[derive(Clone, Copy, Debug)]
pub struct Size<X, Y> {
    pub width: X,
    pub height: Y,
}

impl<X, Y> Size<X, Y>
where
    X: Into<f64>,
    Y: Into<f64>,
{
    pub(crate) fn into_f64(self) -> Size<f64, f64> {
        Size {
            width: self.width.into(),
            height: self.height.into(),
        }
    }
}
