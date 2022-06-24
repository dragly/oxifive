use anyhow::Context;
use clap::Parser;
use std::collections::VecDeque;
use std::{cell::RefCell, fs::File, rc::Rc};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    filename: String,
}

fn main() -> anyhow::Result<()> {
    let Args { filename } = Args::parse();
    let file = File::open(&filename).with_context(|| format!("Could not open `{filename}`"))?;
    let input = Box::new(std::io::BufReader::new(file));
    let mut data = oxifive::read::file::FileReader::read(input)
        .with_context(|| format!("Failed to parse `{filename}`"))?;
    let data_as_group = data.as_group();
    let mut queue = VecDeque::new();
    queue.push_back(Rc::new(RefCell::new(data_as_group)));
    loop {
        let next = match queue.pop_back() {
            None => break,
            Some(v) => v,
        };
        let keys = next.borrow().keys();
        for key in &keys {
            println!("{key:?}");
            let object = next.borrow().object(&mut data, key)?;
            println!("{:#?}", object);
            if object.is_group() {
                queue.push_back(Rc::new(RefCell::new(object.as_group())));
            }
        }
    }
    return Ok(());
}
