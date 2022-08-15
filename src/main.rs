mod hgrm;
mod line;
mod parser;
mod renderer;
mod violin;

use std::io::Read;

use clap::Parser;
use flate2::read;
use parser::parse;
use renderer::{Renderer, RendererInput};

#[macro_use]
extern crate self_update;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    update: bool,

    #[clap(required_unless_present = "update")]
    data: String,

    #[clap(short, long, required_unless_present = "update")]
    filename: Option<String>,

    #[clap(arg_enum, short, long, default_value_t = RendererInput::Line)]
    renderer: RendererInput,
}

fn update() -> Result<(), Box<dyn std::error::Error>> {
    let status = self_update::backends::github::Update::configure()
        .repo_owner("jeffutter")
        .repo_name("hdr2plot")
        .bin_name("hdr2plot")
        .show_download_progress(true)
        .current_version(cargo_crate_version!())
        .build()?
        .update()?;

    println!("Update status: `{}`!", status.version());

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if args.update {
        return update();
    };

    println!("Decoding...");

    let compressed = base64::decode(args.data)?;

    println!("Uncompressing...");

    let mut gz = read::GzDecoder::new(&compressed[..]);
    let mut data = String::new();
    gz.read_to_string(&mut data)?;

    if let Ok((_, parsed)) = parse(&data) {
        let filename = &args.filename.unwrap();
        let renderer = Renderer::new(args.renderer, parsed, filename);

        renderer.render()
    } else {
        println!("Unable to parse data!");
        Ok(())
    }
}
