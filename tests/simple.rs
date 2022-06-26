use ndarray::{array, Ix3};

#[test]
fn read_simple() -> Result<(), oxifive::error::Error> {
    let input = std::fs::File::open("tests/files/simple.h5").unwrap();
    let input = Box::new(input);
    let mut file = oxifive::FileReader::read(input)?;
    let group = file.get("group").expect("group not found");
    let group = group.as_group(&mut file)?;
    let data = group["data"].as_dataset(&mut file)?;
    let array = data.read::<f32, Ix3>(&mut file)?;
    assert!(array == array![[[1.0, 2.0], [8.0, 3.0], [4.0, 9.0]]]);
    Ok(())
}
