
pub struct Message {
    pub header: Vec<u8>,
    pub body: Vec<u8>,
}

impl Message {
    pub fn new() -> Message {
        Message {
            header: Vec::new(),
            body: Vec::new(),
        }
    }
    pub fn construct(header: Vec<u8>, body: Vec<u8>) -> Message {
        Message {
            header: header,
            body: body,
        }
    }
    pub fn len(&self) -> usize {
        self.header.len() + self.body.len()
    }
    pub fn get_header(&self) -> &[u8] {
        &self.header
    }
    pub fn get_body(&self) -> &[u8] {
        &self.body
    }
    pub fn split(self) -> (Vec<u8>, Vec<u8>) {
        (self.header, self.body)
    }
}

impl Into<Vec<u8>> for Message {
    fn into(self) -> Vec<u8> {
        self.body
    }
}

impl From<Vec<u8>> for Message {
    fn from(value: Vec<u8>) -> Message {
        Message::construct(Vec::new(), value)
    }
}
