use crate::hgrm::HGRMs;
use crate::line;
use crate::violin;

#[derive(clap::ArgEnum, Clone, Debug)]
pub enum RendererInput {
    Violin,
    Line,
}

pub enum Renderer<'a> {
    Violin(violin::Violin<'a>),
    Line(line::Line<'a>),
}

impl<'a> Renderer<'a> {
    pub fn new(input: RendererInput, histograms: HGRMs, filename: &'a str) -> Self {
        match input {
            RendererInput::Violin => Renderer::Violin(violin::Violin::new(histograms, filename)),
            RendererInput::Line => Renderer::Line(line::Line::new(histograms, filename)),
        }
    }

    pub fn render(&self) -> Result<(), Box<dyn std::error::Error>> {
        match &self {
            Self::Violin(violin) => violin.render(),
            Self::Line(line) => line.render(),
        }
    }
}
