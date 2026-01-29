pub struct RecvSequenceSpace {
    pub nxt: u32, // receive next
    pub wnd: u16, // receive window
    pub up: u16,  // receive urgent pointer
    pub irs: u32, // initial receive sequence number
}

impl Default for RecvSequenceSpace {
    fn default() -> Self {
        Self {
            nxt: 0,
            wnd: 0,
            up: 0,
            irs: 0,
        }
    }
}
