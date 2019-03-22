//! Types for representing details in the graphics state.

use std::f64::consts::PI;
use std::fmt::{self, Display};
use std::ops::Mul;

/// Line join styles, as described in section 8.4.3.4 of the PDF
/// specification.
#[allow(dead_code)]
pub enum JoinStyle {
    /// The outer edges continues until they meet.
    Miter,
    /// The lines are joined by a circle of line-width diameter.
    Round,
    /// End the lines as with `CapStyle::Butt` and fill the resulting
    /// gap with a triangle.
    Bevel,
}

/// Line cap styles, as described in section 8.4.3.4 of the PDF
/// specification.
#[allow(dead_code)]
pub enum CapStyle {
    /// Truncate the line squarely through the endpoint.
    Butt,
    /// Include a circle of line-width diameter around the endpoint.
    Round,
    /// Include a square around the endpoint, so the line continues for half
    /// a line-width through the endpoint.
    ProjectingSquare,
}

/// Any color (or grayscale) value that this library can make PDF represent.
#[derive(Clone, Copy, Debug)]
#[allow(missing_docs)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl Color {
    /// Return a grayscale color value.

    /// # Example
    /// ````
    /// # use pdfpdf::Color;
    /// let white = Color::gray(255);
    /// let gray = Color::gray(128);
    /// ````
    #[inline]
    pub fn gray(gray: u8) -> Self {
        Self {
            red: gray,
            green: gray,
            blue: gray,
        }
    }
}

/// A transformation matrix for the pdf graphics state.
///
/// Matrices can be created with numerous named constructors and
/// combined by multiplication.
///
/// # Examples
///
/// ```
/// # use pdfpdf::{Matrix, Pdf};
/// Pdf::new()
///     .add_page(180.0, 240.0)
///     .transform(Matrix::translate(10.0, 24.0))
///
/// // Matrixes can be combined by multiplication:
///     .transform(Matrix::translate(7.0, 0.0) * Matrix::rotate_deg(45.0))
/// // ... will be visualy identical to:
///     .transform(Matrix::translate(7.0, 0.0))
///     .transform(Matrix::rotate_deg(45.0))
///     .write_to("foo.pdf").unwrap();
/// ```
#[derive(Clone, Copy, Debug)]
pub struct Matrix {
    v: [f64; 6],
}

impl Matrix {
    /// Construct a matrix for translation
    #[inline]
    pub fn translate<N>(dx: N, dy: N) -> Self
    where
        N: Into<f64>,
    {
        Self {
            v: [1., 0., 0., 1., dx.into(), dy.into()],
        }
    }

    /// Construct a matrix for rotating by `a` radians.
    #[inline]
    pub fn rotate<N>(a: N) -> Self
    where
        N: Into<f64>,
    {
        let a = a.into();
        Self {
            v: [a.cos(), a.sin(), -a.sin(), a.cos(), 0., 0.],
        }
    }

    /// Construct a matrix for rotating by `a` degrees.
    #[inline]
    pub fn rotate_deg<N>(a: N) -> Self
    where
        N: Into<f64>,
    {
        Self::rotate(a.into() * PI / 180.)
    }

    /// Construct a matrix for scaling by factor `sx` in x-direction
    /// and by `sy` in y-direction.
    #[inline]
    pub fn scale<N>(sx: N, sy: N) -> Self
    where
        N: Into<f64>,
    {
        Self {
            v: [sx.into(), 0., 0., sy.into(), 0., 0.],
        }
    }

    /// Construct a matrix for scaling by the same factor, `s` in both
    /// directions.
    #[inline]
    pub fn uniform_scale<N>(s: N) -> Self
    where
        N: Into<f64> + Clone,
    {
        Self::scale(s.clone().into(), s.into())
    }

    /// Construct a matrix for skewing.
    #[inline]
    pub fn skew<N>(a: N, b: N) -> Self
    where
        N: Into<f64>,
    {
        Self {
            v: [1., a.into().tan(), b.into().tan(), 1., 0., 0.],
        }
    }
}

impl Display for Matrix {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let v = self.v;
        write!(f, "{} {} {} {} {} {}", v[0], v[1], v[2], v[3], v[4], v[5])
    }
}

impl Mul for Matrix {
    type Output = Self;
    #[inline]
    fn mul(self, b: Self) -> Self {
        let a = self.v;
        let b = b.v;
        Self {
            v: [
                a[0] * b[0] + a[1] * b[2],
                a[0] * b[1] + a[1] * b[3],
                a[2] * b[0] + a[3] * b[2],
                a[2] * b[1] + a[3] * b[3],
                a[4] * b[0] + a[5] * b[2] + b[4],
                a[4] * b[1] + a[5] * b[3] + b[5],
            ],
        }
    }
}

#[test]
fn test_matrix_mul_a() {
    assert_unit(&(Matrix::rotate_deg(45.) * Matrix::rotate_deg(-45.)));
}

#[test]
fn test_matrix_mul_b() {
    assert_unit(&(Matrix::uniform_scale(2.) * Matrix::uniform_scale(0.5)));
}

#[test]
fn test_matrix_mul_c() {
    assert_unit(&Matrix::rotate(2. * PI));
}

#[test]
fn test_matrix_mul_d() {
    assert_unit(&(Matrix::rotate(PI) * Matrix::uniform_scale(-1.)));
}

#[cfg(test)]
fn assert_unit(m: &Matrix) {
    assert_eq!(None, diff(&[1., 0., 0., 1., 0., 0.], &m.v));
}

#[cfg(test)]
fn diff(a: &[f64; 6], b: &[f64; 6]) -> Option<String> {
    let large_a = a.iter().fold(0_f64, |x, &y| x.max(y));
    let large_b = b.iter().fold(0_f64, |x, &y| x.max(y));
    let epsilon = 1e-6 * large_a.max(large_b);
    for i in 0..6 {
        if (a[i] - b[i]).abs() > epsilon {
            return Some(format!("{:?} != {:?}", a, b));
        }
    }
    None
}
