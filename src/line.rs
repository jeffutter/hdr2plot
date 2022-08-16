use plotters::prelude::*;

use crate::hgrm::HGRMs;

pub struct Line<'a> {
    histograms: HGRMs,
    filename: &'a str,
}

impl<'a> Line<'a> {
    pub fn new(histograms: HGRMs, filename: &'a str) -> Self {
        Self {
            histograms,
            filename,
        }
    }

    pub fn render(&self) -> Result<(), Box<dyn std::error::Error>> {
        let size = (960, 480);
        let root = SVGBackend::new(self.filename, size).into_drawing_area();

        let mut chart = ChartBuilder::on(&root)
            .caption("Latency", ("sans-serif", 30).into_font())
            .margin(5)
            .x_label_area_size(35)
            .y_label_area_size(60)
            .build_cartesian_2d(
                (0f64..1f64)
                    .log_scale()
                    .zero_point(1.0)
                    .with_key_points(vec![0.9999, 0.999, 0.99, 0.95, 0.9, 0.5, 0.1]),
                0f64..(self.histograms.max_latency() / 1000f64),
            )?;

        chart
            .configure_mesh()
            .x_desc("Percentile")
            .x_label_formatter(&|x| format!("{}%", *x * 100.0))
            .y_desc("Milliseconds")
            .y_max_light_lines(5)
            .draw()?;

        for (idx, histogram) in self.histograms.iter().enumerate() {
            let color = Palette99::pick(idx);

            let mut data = histogram
                .percentiles
                .iter()
                .filter(|percentile| percentile.percentile < 1.0f64)
                .map(|percentile| (percentile.percentile, percentile.value / 1000f64));

            let label = match &histogram.name {
                Some(filename) => {
                    format!("{}, {} Total", filename, histogram.total_count)
                }
                None => {
                    format!("{} Total", histogram.total_count)
                }
            };

            chart
                .draw_series(LineSeries::new(&mut data, &color))?
                .label(label)
                .legend(move |(x, y)| {
                    let color = Palette99::pick(idx);
                    PathElement::new(vec![(x, y), (x + 20, y)], color)
                });
        }

        chart
            .configure_series_labels()
            .background_style(&WHITE.mix(0.8))
            .border_style(&BLACK)
            .draw()?;

        Ok(())
    }
}
