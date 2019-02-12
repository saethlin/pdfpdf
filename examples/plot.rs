extern crate pdfpdf;
use pdfpdf::{Alignment::*, Color, Matrix, Pdf};
use std::io;

struct Plot {
    pdf: Pdf,
    width: f64,
    height: f64,
    x_tick_interval: Option<f64>,
    y_tick_interval: Option<f64>,
    xlim: Option<(f64, f64)>,
    ylim: Option<(f64, f64)>,
}

#[derive(Debug, Clone, Copy)]
struct Point {
    x: f64,
    y: f64,
}

impl Plot {
    pub fn new() -> Self {
        Self {
            pdf: Pdf::new_uncompressed(),
            width: 500.0,
            height: 500.0,
            x_tick_interval: None,
            y_tick_interval: None,
            xlim: None,
            ylim: None,
        }
    }

    pub fn plot(&mut self, points: &[Point]) -> &mut Self {
        let x_tick_interval = self.x_tick_interval.unwrap();
        let y_tick_interval = self.y_tick_interval.unwrap();
        let xlim = self.xlim.unwrap();
        let ylim = self.ylim.unwrap();

        let border = 0.075f64;
        // Draw the plot's border
        self.pdf
            .add_page(self.width, self.height)
            .set_color(Color::gray(0))
            .set_line_width(0.75)
            .move_to(self.width * (1.0 - border), self.width * border)
            .line_to(self.width * border, self.height * border)
            .line_to(self.width * border, self.height * (1.0 - border))
            .end_line();

        let (min, max) = {
            use std::f64;
            let mut max = Point {
                x: f64::NEG_INFINITY,
                y: f64::NEG_INFINITY,
            };
            let mut min = Point {
                x: f64::INFINITY,
                y: f64::INFINITY,
            };
            for p in points {
                max.x = max.x.max(p.x);
                max.y = max.y.max(p.y);
                min.x = min.x.min(p.x);
                min.y = min.y.min(p.y);
            }
            (min, max)
        };

        assert!(max.x.is_finite());
        assert!(max.y.is_finite());
        assert!(min.x.is_finite());
        assert!(min.y.is_finite());

        let x_scale = self.width * (1.0 - 2.0 * border) / (max.x - min.x);
        let y_scale = self.width * (1.0 - 2.0 * border) / (max.x - min.x);

        let range = max.x;
        println!("{}", range);
        let order_of_magnitude = (10.0f64).powi(range.log10() as i32);
        let possible_tick_intervals = [
            order_of_magnitude / 2.0,
            order_of_magnitude,
            order_of_magnitude * 2.0,
        ];
        let num_ticks = [
            (range / possible_tick_intervals[0]).round() as i64,
            (range / possible_tick_intervals[1]).round() as i64,
            (range / possible_tick_intervals[2]).round() as i64,
        ];
        // Try to get as close to 5 ticks as possible
        let chosen_index = num_ticks
            .iter()
            .enumerate()
            .min_by_key(|(_, num)| (**num - 5).abs())
            .unwrap()
            .0;
        let tick_interval = possible_tick_intervals[chosen_index];
        let num_ticks = num_ticks[chosen_index];

        println!("{:?}", tick_interval);

        let plot_height = self.height * (1.0 - 2. * border);
        let plot_width = self.width * (1.0 - 2. * border);

        let x_scale = plot_width / max.x;
        let y_scale = plot_height / max.y;
        let scaled: Vec<_> = points
            .iter()
            .map(|p| {
                (
                    (p.x * x_scale) + (border * self.width),
                    (p.y * y_scale) + (border * self.height),
                )
            })
            .collect();

        // draw the tick marks
        for i in 0..num_ticks {
            let x = i as f64 * tick_interval;
            let plot_x = x * x_scale + border * self.width;
            self.pdf
                .move_to(x * x_scale + border * self.width, self.height * border)
                .line_to(
                    x * x_scale + border * self.width,
                    self.height * border * 0.8,
                )
                .end_line();
            self.pdf.draw_text(
                plot_x,
                self.height * border * 0.8,
                TopCenter,
                &format!("{}", x),
            );
        }

        // Draw the data series
        self.pdf
            .set_line_width(1.5)
            .set_color(Color::rgb(31, 119, 180))
            .draw_line(scaled.iter())
            // Draw the x label
            .set_color(Color::gray(0))
            .draw_text(self.width / 2., 2.0, BottomCenter, "xlabel")
            // Draw the y label
            .transform(Matrix::rotate_deg(90))
            .draw_text(self.height / 2., 0, TopCenter, "ylabel");

        self
    }

    pub fn write_to(&mut self, filename: &str) -> io::Result<()> {
        self.pdf.write_to(filename)
    }
}

fn main() {
    let x = (0..4096).map(|n| n as f64 / 4095. * 600.);
    let y = x
        .clone()
        .map(|x| (-(x - 300.0).powi(2) / 1200.0).exp() * 600.0);
    let points: Vec<_> = x.zip(y).map(|(x, y)| Point { x, y }).collect();
    let mut plot = Plot::new();
    plot.xlim = Some((0.0, 600.0));
    plot.ylim = Some((0.0, 600.0));
    plot.x_tick_interval = Some(100.0);
    plot.y_tick_interval = Some(100.0);

    plot.plot(&points).write_to("plot.pdf").unwrap();
}
