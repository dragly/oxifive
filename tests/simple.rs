use std::{rc::Rc, cell::RefCell, io::Read, sync::{Arc, Mutex}};

use ndarray::{array, Ix3};

#[test]
fn read_simple() -> Result<(), oxifive::error::Error> {
    let input = std::fs::File::open("tests/files/simple.h5").unwrap();
    let file = oxifive::FileReader::new(input)?;
    let group = file.group("group")?;
    let data = group.dataset("data")?;
    let array = data.read::<f32, Ix3>()?;
    assert!(array == array![[[1.0, 2.0], [8.0, 3.0], [4.0, 9.0]]]);
    Ok(())
}
