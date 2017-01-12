#[derive(Debug)]
pub struct Spawn {
    pub id: u32,
    pub id2: usize,
}
#[derive(Debug)]
pub enum Message {
    Spawn(Spawn),
    Halt,
    Unknown,
}

impl Message {
    pub fn from_u8(b: &[u8]) -> Self {
        //
        Message::Unknown
    }
}