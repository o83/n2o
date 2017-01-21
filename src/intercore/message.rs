// #[derive(Debug,Clone,Copy)]
// pub struct TaskId(usize);
use queues::publisher::Subscriber;

#[derive(Debug,Clone)]
pub struct Pub {
    pub from: usize,
    pub to: usize,
    pub task_id: usize,
    pub name: String,
}

#[derive(Debug,Clone)]
pub struct Sub {
    pub from: usize,
    pub to: usize,
    pub task_id: usize,
    pub pub_id: usize,
}

#[derive(Debug,Clone)]
pub struct Spawn {
    pub from: usize,
    pub to: usize,
    pub txt: String,
}

#[derive(Debug,Clone)]
pub struct AckSub {
    pub from: usize,
    pub to: usize,
    pub task_id: usize,
    pub result_id: usize,
    pub subs: Subscriber<Message>,
}

#[derive(Debug,Clone)]
pub struct AckPub {
    pub from: usize,
    pub to: usize,
    pub task_id: usize,
    pub result_id: usize,
}

#[derive(Debug,Clone)]
pub struct AckSpawn {
    pub from: usize,
    pub to: usize,
    pub task_id: usize,
}

#[derive(Debug,Clone)]
pub enum Message {
    Pub(Pub),
    Sub(Sub),
    Print(String),
    Spawn(Spawn),
    AckSub(AckSub),
    AckPub(AckPub),
    AckSpawn(AckSpawn),
    Select(String,u16),
    Halt,
    Unknown,
}

impl Message {
    pub fn from_u8(b: &[u8]) -> Self {
        //
        Message::Unknown
    }
}