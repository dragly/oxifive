use anyhow::Context;
use clap::Parser;
use eframe::egui;
use oxifive::{Group, Object};
use std::collections::VecDeque;
use std::marker::PhantomData;
use std::{cell::RefCell, fs::File, rc::Rc};

struct QueueElement<R> {
    name: String,
    group: Group<R>,
    node: Handle<Node>,
}

#[derive(Debug)]
struct Arena<T> {
    data: Vec<T>,
}

impl<T> Arena<T> {
    pub fn new() -> Arena<T> {
        Arena { data: vec![] }
    }
    pub fn append(&mut self, value: T) -> Handle<T> {
        self.data.push(value);
        Handle::new(self.data.len() - 1)
    }
    pub fn get(&self, handle: &Handle<T>) -> &T {
        self.data
            .get(handle.index)
            .expect("Item not found for handle `{handle:?}`")
    }
    pub fn get_mut(&mut self, handle: &Handle<T>) -> &mut T {
        self.data
            .get_mut(handle.index)
            .expect("Item not found for handle `{handle:?}`")
    }
}

#[derive(Clone, Copy, Debug)]
struct Handle<T> {
    index: usize,
    _phantom: PhantomData<T>,
}

impl<T> Handle<T> {
    pub fn new(index: usize) -> Handle<T> {
        Handle {
            index,
            _phantom: PhantomData,
        }
    }
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    filename: String,
}

#[derive(Clone, Debug)]
enum Node {
    Group {
        name: String,
        children: Vec<Handle<Node>>,
    },
    Dataset {
        name: String,
    },
}

struct MyApp {
    filename: String,
    nodes: Arena<Node>,
    root_node: Handle<Node>,
}

fn main() -> anyhow::Result<()> {
    let Args { filename } = Args::parse();
    let options = eframe::NativeOptions::default();
    let file = File::open(&filename).with_context(|| format!("Could not open `{filename}`"))?;
    let input = std::io::BufReader::new(file);
    let data = oxifive::read::file::FileReader::new(input)
        .with_context(|| format!("Failed to parse `{filename}`"))?;
    let data_as_group = data.as_group();
    let mut queue = VecDeque::new();
    let mut nodes = Arena::new();
    let root_node = nodes.append(Node::Group {
        name: "/".into(),
        children: vec![],
    });
    queue.push_back(Rc::new(RefCell::new(QueueElement {
        name: filename.clone(),
        group: data_as_group,
        node: root_node.clone(),
    })));
    loop {
        let next = match queue.pop_back() {
            None => break,
            Some(v) => v,
        };
        let mut keys = next.borrow().group.keys();
        keys.sort_by(|a, b| b.cmp(a));
        for key in &keys {
            let object = next.borrow().group.object(key)?;
            let node = match object {
                Object::Group(group) => {
                    let node = nodes.append(Node::Group {
                        name: key.clone(),
                        children: vec![],
                    });
                    queue.push_back(Rc::new(RefCell::new(QueueElement {
                        name: key.clone(),
                        group,
                        node: node.clone(),
                    })));
                    node
                }
                Object::Dataset(dataset) => {
                    let shape = dataset.shape();
                    let datatype = dataset.datatype();
                    let size = datatype.size * 4;
                    let encoding = datatype.encoding;
                    let node = nodes.append(Node::Dataset { name: key.clone() });
                    node
                }
            };
            match nodes.get_mut(&next.borrow_mut().node) {
                Node::Group { children, .. } => {
                    children.push(node.clone());
                }
                _ => panic!("Queued node cannot be a dataset"),
            };
        }
    }
    eframe::run_native(
        "My egui App",
        options,
        Box::new(|_cc| {
            Box::new(MyApp {
                filename,
                nodes,
                root_node,
            })
        }),
    )
}

fn draw_node(ui: &mut egui::Ui, node: &Handle<Node>, nodes: &Arena<Node>) {
    match nodes.get(node) {
        Node::Group { name, children } => {
            if !children.is_empty() {
                ui.collapsing(format!("{name}"), |ui| {
                    for child in children {
                        draw_node(ui, child, nodes);
                    }
                });
            }
        }
        Node::Dataset { name } => {
            ui.label(format!("{name}"));
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(format!("{filename}", filename = self.filename));
            draw_node(ui, &self.root_node, &self.nodes);
        });
    }
}
