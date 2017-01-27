
#[derive(PartialEq, Debug,Clone)]
pub struct Pub {
    pub from: usize,
    pub to: usize,
    pub task_id: usize,
    pub name: String,
    pub cap: usize,
}

#[derive(PartialEq, Debug,Clone)]
pub struct Sub {
    pub from: usize,
    pub to: usize,
    pub task_id: usize,
    pub pub_id: usize,
}

#[derive(PartialEq, Debug,Clone)]
pub struct Spawn {
    pub from: usize,
    pub to: usize,
    pub txt: String,
}

#[derive(PartialEq, Debug,Clone)]
pub struct AckSub {
    pub from: usize,
    pub to: usize,
    pub task_id: usize,
    pub result_id: usize, //    pub s: Subscriber<Message>,
}

#[derive(PartialEq, Debug,Clone)]
pub struct AckPub {
    pub from: usize,
    pub to: usize,
    pub task_id: usize,
    pub result_id: usize,
}

#[derive(PartialEq, Debug,Clone)]
pub struct AckSpawn {
    pub from: usize,
    pub to: usize,
    pub task_id: usize,
}

#[derive(PartialEq, Debug,Clone)]
pub enum Message {
    Pub(Pub),
    Sub(Sub),
    Print(String),
    Spawn(Spawn),
    AckSub(AckSub),
    AckPub(AckPub),
    AckSpawn(AckSpawn),
    Exec(usize, String),
    Select(String, u16),
    QoS(u8, u8, u8),
    Halt,
    Nop,
}

impl Message {
    pub fn from_u8(b: &[u8]) -> Self {
        //
        Message::Nop
    }
}
