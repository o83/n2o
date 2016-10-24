
use std::rc::Rc;
use std::io::{self, ErrorKind, Error};
use std::ptr::copy_nonoverlapping;
use core::mem::transmute;
use network::message::Message;
use network::tcp::pipe::WriteBuffer;

pub struct SendOperation {
    step: Option<SendOperationStep>,
}

impl SendOperation {
    pub fn new(msg: Rc<Message>) -> SendOperation {
        SendOperation { step: Some(SendOperationStep::TransportHdr(msg, 0)) }
    }

    pub fn run<T: io::Write>(&mut self, stream: &mut T) -> io::Result<bool> {
        if let Some(step) = self.step.take() {
            self.resume_at(stream, step)
        } else {
            Err(Error::new(ErrorKind::Other,
                           "Cannot resume already finished send operation"))
        }
    }

    fn resume_at<T: io::Write>(&mut self,
                               stream: &mut T,
                               step: SendOperationStep)
                               -> io::Result<bool> {
        let mut cur_step = step;

        loop {
            let (passed, next_step) = try!(cur_step.advance(stream));

            if next_step.is_terminal() {
                return Ok(true);
            }
            if !passed {
                self.step = Some(next_step);
                return Ok(false);
            }

            cur_step = next_step;
        }
    }
}

enum SendOperationStep {
    TransportHdr(Rc<Message>, usize),
    ProtocolHdr(Rc<Message>, usize),
    UsrPayload(Rc<Message>, usize),
    Terminal,
}

impl SendOperationStep {
    fn advance<T: io::Write>(self, stream: &mut T) -> io::Result<(bool, SendOperationStep)> {
        match self {
            SendOperationStep::TransportHdr(msg, written) => {
                write_transport_hdr(stream, msg, written)
            }
            SendOperationStep::ProtocolHdr(msg, written) => {
                write_protocol_hdr(stream, msg, written)
            }
            SendOperationStep::UsrPayload(msg, written) => write_usr_payload(stream, msg, written),
            SendOperationStep::Terminal => {
                Err(Error::new(ErrorKind::Other,
                               "Cannot advance terminal step of send operation"))
            }
        }
    }

    fn is_terminal(&self) -> bool {
        match *self {
            SendOperationStep::Terminal => true,
            _ => false,
        }
    }
}

fn write_transport_hdr<T: io::Write>(stream: &mut T,
                                     msg: Rc<Message>,
                                     mut written: usize)
                                     -> io::Result<(bool, SendOperationStep)> {
    let msg_len = msg.len() as u64;
    let mut buffer = [0u8; 8];
    // BigEndian::write_u64(&mut buffer, msg_len);
    unsafe {
        let bytes = transmute::<_, [u8; 8]>(msg_len.to_be());
        copy_nonoverlapping((&bytes).as_ptr(), (&mut buffer[0..4]).as_mut_ptr(), 4);
    }
    let sent = try!(stream.write_buffer(&buffer, &mut written));
    if sent {
        Ok((true, SendOperationStep::ProtocolHdr(msg, 0)))
    } else {
        Ok((false, SendOperationStep::TransportHdr(msg, written)))
    }
}

fn write_protocol_hdr<T: io::Write>(stream: &mut T,
                                    msg: Rc<Message>,
                                    mut written: usize)
                                    -> io::Result<(bool, SendOperationStep)> {
    if msg.get_header().len() == 0 {
        return Ok((true, SendOperationStep::UsrPayload(msg, 0)));
    }

    let sent = try!(stream.write_buffer(msg.get_header(), &mut written));
    if sent {
        Ok((true, SendOperationStep::UsrPayload(msg, 0)))
    } else {
        Ok((false, SendOperationStep::ProtocolHdr(msg, written)))
    }
}

fn write_usr_payload<T: io::Write>(stream: &mut T,
                                   msg: Rc<Message>,
                                   mut written: usize)
                                   -> io::Result<(bool, SendOperationStep)> {
    if msg.get_body().len() == 0 {
        return Ok((true, SendOperationStep::Terminal));
    }

    let sent = try!(stream.write_buffer(msg.get_body(), &mut written));
    if sent {
        Ok((true, SendOperationStep::Terminal))
    } else {
        Ok((false, SendOperationStep::UsrPayload(msg, written)))
    }
}

#[cfg(test)]
mod tests {
    // use std::ops::Deref;
    // use std::rc::Rc;
    //
    // use network::message::Message;
    // use network::tcp::send::SendOperation;
    //
    // #[test]
    // fn send_in_one_run() {
    // let header = vec![1, 4, 3, 2];
    // let payload = vec![65, 66, 67, 69];
    // let msg = Message::construct(header, payload);
    // let mut operation = SendOperation::new(Rc::new(msg));
    // let mut stream = Vec::new();
    // let result = operation.run(&mut stream).expect("send should have succeeded");
    // let expected_bytes = [0u8, 0, 0, 0, 0, 0, 0, 8, 1, 4, 3, 2, 65, 66, 67, 69];
    //
    // assert!(result);
    // assert_eq!(&expected_bytes, stream.deref());
    // }
    //
}