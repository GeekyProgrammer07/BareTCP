// if self.send.una < seg_ack {
//     if self.send.nxt >= self.send.una && self.send.nxt < seg_ack {
//         // Invalid Position for N
//         return Ok(());
//     }
// } else {
//     if self.send.nxt >= seg_ack && self.send.nxt < self.send.una {
//         // Accepted only here
//     } else {
//         return Ok(());
//     }
// }

// Thse are kind of same as above
pub fn seq_lt(a: u32, b: u32) -> bool {
    (a.wrapping_sub(b) as i32) < 0
}

pub fn seq_le(a: u32, b: u32) -> bool {
    (a.wrapping_sub(b) as i32) <= 0
}
