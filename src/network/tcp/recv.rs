
use std::io::{self, ErrorKind, Error};
use network::message::Message;
use std::ptr::copy_nonoverlapping;
use network::tcp::pipe::ReadBuffer;

pub struct RecvOperation {
    step: Option<RecvOperationStep>,
}

impl RecvOperation {
    pub fn new(recv_max_size: u64) -> RecvOperation {
        RecvOperation { step: Some(RecvOperationStep::Header([0; 8], 0, recv_max_size)) }
    }

    pub fn run<T: io::Read>(&mut self, stream: &mut T) -> io::Result<Option<Message>> {
        if let Some(step) = self.step.take() {
            self.resume_at(stream, step)
        } else {
            Err(Error::new(ErrorKind::Other,
                           "Cannot resume already finished recv operation"))
        }
    }

    fn resume_at<T: io::Read>(&mut self,
                              stream: &mut T,
                              step: RecvOperationStep)
                              -> io::Result<Option<Message>> {
        let mut cur_step = step;

        loop {
            let (passed, next_step) = try!(cur_step.advance(stream));

            if !passed {
                self.step = Some(next_step);
                return Ok(None);
            }

            match next_step {
                RecvOperationStep::Terminal(msg) => return Ok(Some(msg)),
                other => cur_step = other,
            }
        }
    }
}

enum RecvOperationStep {
    Header([u8; 8], usize, u64),
    Payload(Vec<u8>, usize),
    Terminal(Message),
}

impl RecvOperationStep {
    fn advance<T: io::Read>(self, stream: &mut T) -> io::Result<(bool, RecvOperationStep)> {
        match self {
            RecvOperationStep::Header(buffer, read, max_size) => {
                read_header(stream, buffer, read, max_size)
            }
            RecvOperationStep::Payload(buffer, read) => read_payload(stream, buffer, read),
            RecvOperationStep::Terminal(_) => {
                Err(Error::new(ErrorKind::Other,
                               "Cannot advance terminal step of recv operation"))
            }
        }
    }
}

fn read_header<T: io::Read>(stream: &mut T,
                            mut buffer: [u8; 8],
                            mut read: usize,
                            max_size: u64)
                            -> io::Result<(bool, RecvOperationStep)> {
    read += try!(stream.read_buffer(&mut buffer[read..]));

    if read == 8 {
        let msg_len = unsafe {
            let mut data: u64 = 0;
            copy_nonoverlapping((&buffer).as_ptr(), &mut data as *mut u64 as *mut u8, 8);
            data.to_be()
        };
        // BigEndian::read_u64(&buffer);
        // read_num_bytes!(u64, 8, buf, to_be)
        if msg_len > max_size {
            Err(Error::new(ErrorKind::Other, "message is too long"))
        } else {
            let payload = vec![0u8; msg_len as usize];

            Ok((true, RecvOperationStep::Payload(payload, 0)))
        }
    } else {
        Ok((false, RecvOperationStep::Header(buffer, read, max_size)))
    }
}

fn read_payload<T: io::Read>(stream: &mut T,
                             mut buffer: Vec<u8>,
                             mut read: usize)
                             -> io::Result<(bool, RecvOperationStep)> {
    read += try!(stream.read_buffer(&mut buffer[read..]));

    if read == buffer.capacity() {
        Ok((true, RecvOperationStep::Terminal(Message::construct(Vec::new(), buffer))))
    } else {
        Ok((false, RecvOperationStep::Payload(buffer, read)))
    }
}
