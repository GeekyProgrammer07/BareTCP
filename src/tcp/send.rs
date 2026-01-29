pub struct SendSequenceSpace {
    pub una: u32,        // send unacknowledged
    pub nxt: u32,        // send next
    pub wnd: u16,        // send window
    pub up: Option<u16>, // send urgent pointer
    pub wl1: u32,        // segment sequence number used for last window update
    pub wl2: u32,        // segment acknowledgment number used for last window update
    pub iss: u32,        // initial send sequence number
}

impl Default for SendSequenceSpace {
    fn default() -> Self {
        Self {
            una: 0,
            nxt: 0,
            wnd: 0,
            up: None,
            wl1: 0,
            wl2: 0,
            iss: 0,
        }
    }
}
