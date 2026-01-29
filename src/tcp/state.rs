#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum State {
    Closed,
    Listen,
    SynRcvd,
    Estab,
}
