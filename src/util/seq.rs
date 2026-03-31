pub fn seq_lt(a: u32, b: u32) -> bool {
    (a.wrapping_sub(b) as i32) < 0
}

pub fn seq_le(a: u32, b: u32) -> bool {
    (a.wrapping_sub(b) as i32) <= 0
}
