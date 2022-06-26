use anyhow::Context;
use clap::Parser;
use oxifive::{Group, Object};
use std::collections::VecDeque;
use std::{cell::RefCell, fs::File, rc::Rc};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    filename: String,
}

struct QueueElement {
    name: String,
    group: Group,
    indentation: usize,
}

fn main() -> anyhow::Result<()> {
    let Args { filename } = Args::parse();
    let file = File::open(&filename).with_context(|| format!("Could not open `{filename}`"))?;
    let input = Box::new(std::io::BufReader::new(file));
    let mut data = oxifive::read::file::FileReader::read(input)
        .with_context(|| format!("Failed to parse `{filename}`"))?;
    let data_as_group = data.as_group();
    let mut queue = VecDeque::new();
    queue.push_back(Rc::new(RefCell::new(QueueElement {
        name: filename,
        group: data_as_group,
        indentation: 1,
    })));
    loop {
        let next = match queue.pop_back() {
            None => break,
            Some(v) => v,
        };
        let name = &next.borrow().name;
        let indentation = next.borrow().indentation;
        println!("{:->indentation$} {name}", "");
        let mut keys = next.borrow().group.keys();
        keys.sort_by(|a, b| b.cmp(a));
        for key in &keys {
            let object = next.borrow().group[key].follow(&mut data)?;
            match object {
                Object::Group(group) => {
                    queue.push_back(Rc::new(RefCell::new(QueueElement {
                        name: key.clone(),
                        group,
                        indentation: indentation + 1,
                    })));
                }
                Object::Dataset(dataset) => {
                    let dataset_indentation = indentation + 1;
                    let shape = dataset.shape();
                    let datatype = dataset.datatype();
                    let size = datatype.size * 4;
                    let encoding = datatype.encoding;
                    println!("{:->dataset_indentation$} {key} [dataset shape {shape:?} type {encoding:?} size {size}]", "");
                }
            }
        }
    }
    return Ok(());
}
