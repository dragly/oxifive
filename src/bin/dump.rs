use std::fs::File;
use anyhow::Context;
use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    filename: String,
}

fn main() -> anyhow::Result<()> {
    let Args { filename } = Args::parse();
    let file = File::open(&filename).with_context(|| format!("Could not open `{filename}`"))?;
    let input = Box::new(std::io::BufReader::new(file));
    let mut data = oxifive::read::file::FileReader::read(input).with_context(|| format!("Failed to parse `{filename}`"))?;
    let keys = data.keys();
    for key in &keys {
        println!("{key:?}");
        println!("{:#?}", data.object(key));
    }
    return Ok(())
}

