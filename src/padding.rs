pub fn padded_size(size: usize) -> usize {
    let padding = 8;
    ((size as f64 / padding as f64).ceil() as usize) * padding
}
