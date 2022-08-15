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
        let size = (960, 720);
        let root = SVGBackend::new(self.filename, size).into_drawing_area();

        let mut chart = ChartBuilder::on(&root)
            .caption("Latency", ("sans-serif", 30).into_font())
            .margin(5)
            .x_label_area_size(35)
            .y_label_area_size(60)
            .build_cartesian_2d(
                self.histograms.min_pct()..self.histograms.max_pct(),
                0f64..self.histograms.max_latency(),
            )?;

        chart
            .configure_mesh()
            .x_desc("Percentile")
            .y_desc("Milliseconds")
            .draw()?;

        for (idx, histogram) in self.histograms.iter().enumerate() {
            let color = Palette99::pick(idx);

            let mut data = histogram
                .percentiles
                .iter()
                .map(|percentile| (percentile.percentile * 100.0, percentile.value));

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
