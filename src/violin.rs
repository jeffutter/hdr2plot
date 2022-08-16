use csaps::CubicSmoothingSpline;
use itertools::Itertools;
use plotters::prelude::*;

use crate::hgrm::HGRMs;

const DARK_BLUE: RGBColor = RGBColor(31, 120, 180);

pub struct Violin<'a> {
    histograms: HGRMs,
    filename: &'a str,
}

impl<'a> Violin<'a> {
    pub fn new(histograms: HGRMs, filename: &'a str) -> Self {
        Self {
            histograms,
            filename,
        }
    }

    pub fn render(&self) -> Result<(), Box<dyn std::error::Error>> {
        let histograms = &self.histograms;
        let filen = self.filename;
        let x_range = 0.0..(histograms.max_latency() / 1000f64);
        let y_range = -0.5..histograms.len() as f64 - 0.5;

        let size = (960, 300 + (18 * histograms.len() as u32));
        let root = SVGBackend::new(filen, size).into_drawing_area();

        let mut chart = ChartBuilder::on(&root)
            .caption("Latency", ("sans-serif", 30).into_font())
            .margin((5).percent())
            .set_label_area_size(LabelAreaPosition::Left, (10).percent_width().min(60))
            .set_label_area_size(LabelAreaPosition::Bottom, (5).percent_width().min(40))
            .build_cartesian_2d(x_range, y_range)?;

        let y_label_formatter = |v: &f64| {
            let histogram = &histograms[v.round() as usize];

            match &histogram.name {
                Some(filename) => {
                    format!("{}, {} Total", filename, histogram.total_count)
                }
                None => {
                    format!("{} Total", histogram.total_count)
                }
            }
        };

        chart
            .configure_mesh()
            .disable_mesh()
            .y_desc("Input")
            .x_desc("Latency (ms)")
            .y_label_formatter(&y_label_formatter)
            .y_labels(histograms.len())
            .x_label_formatter(&|v: &f64| (v.round() as usize).to_string())
            .draw()?;

        for (idx, histogram) in histograms.iter().enumerate() {
            let base = idx as f64;

            let histogram_max_y = histogram
                .percentiles
                .iter()
                .scan(0f64, |prev, percentile| {
                    let count_diff = (percentile.total_count as f64) - *prev;
                    *prev = percentile.total_count as f64;

                    Some(count_diff)
                })
                .fold(f64::MIN, |a, b| a.max(b));

            let scaler = |i: f64| (i / histogram_max_y) * 0.9;

            let mut data: Vec<(f64, f64)> = histogram
                .percentiles
                .iter()
                .scan(0f64, |prev, percentile| {
                    let count_diff = (percentile.total_count as f64) - *prev;
                    *prev = percentile.total_count as f64;

                    Some((percentile.value / 1000f64, count_diff))
                })
                .group_by(|(x, _y)| *x)
                .into_iter()
                .map(|(x, ys)| {
                    let s = ys.into_iter().map(|(_, ys)| ys).sum();

                    (x, scaler(s))
                })
                .collect();

            data.sort_by(|(x1, _), (x2, _)| x1.partial_cmp(x2).unwrap());

            let xs: Vec<f64> = data.iter().map(|(x, _y)| *x).collect();
            let ys: Vec<f64> = data.iter().map(|(_x, y)| *y).collect();

            let smooth_ys = CubicSmoothingSpline::new(&xs, &ys)
                .with_smooth(0.99)
                .make()
                .unwrap()
                .evaluate(&xs)
                .unwrap();

            let smoothdata: Vec<(f64, f64)> = xs
                .iter()
                .zip(smooth_ys.iter())
                .map(|(x, y)| (*x, *y))
                .collect();

            chart.draw_series(AreaSeries::new(
                smoothdata.iter().map(|(x, y)| (*x, base + *y / 2.0)),
                base,
                &DARK_BLUE, // Palette99::pick(idx),
            ))?;

            chart.draw_series(AreaSeries::new(
                smoothdata.iter().map(|(x, y)| (*x, base - *y / 2.0)),
                base,
                &DARK_BLUE, // Palette99::pick(idx),
            ))?;
        }

        Ok(())
    }
}
