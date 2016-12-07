
//  Socket Connection Stream by 5HT

use std::io::{self, ErrorKind, Error};
use std::rc::Rc;
use io::token::Token;
use io::ready::Ready;
use io::poll::*;
use io::options::*;
use io::tcp::TcpStream;
use slab;
use core::mem::transmute;
use core::ptr::copy_nonoverlapping;
use std::io::prelude::*;

pub struct Connection {
    sock: TcpStream,
    pub token: Token,
    interest: Ready,
    send_queue: Vec<Rc<Vec<u8>>>,
    is_idle: bool,
    is_reset: bool,
    read_continuation: Option<u64>,
    write_continuation: bool,
}

impl Connection {
    pub fn new(sock: TcpStream, token: Token) -> Connection {
        Connection {
            sock: sock,
            token: token,
            interest: Ready::hup(),
            send_queue: Vec::new(),
            is_idle: true,
            is_reset: false,
            read_continuation: None,
            write_continuation: false,
        }
    }

    pub fn readable(&mut self) -> io::Result<Option<Vec<u8>>> {

        let msg_len = match try!(self.read_message_length()) {
            None => {
                return Ok(None);
            }
            Some(n) => n,
        };

        if msg_len == 0 {
            println!("message is zero bytes; token={:?}", self.token);
            return Ok(None);
        }

        let msg_len = msg_len as usize;

        println!("Expected message length is {}", msg_len);
        let mut recv_buf: Vec<u8> = Vec::with_capacity(msg_len);
        unsafe {
            recv_buf.set_len(msg_len);
        }

        // UFCS: resolve "multiple applicable items in scope [E0034]" error
        let sock_ref = <TcpStream as Read>::by_ref(&mut self.sock);

        match sock_ref.take(msg_len as u64).read(&mut recv_buf) {
            Ok(n) => {
                println!("CONN : we read {} bytes", n);

                if n < msg_len as usize {
                    return Err(Error::new(ErrorKind::InvalidData, "Did not read enough bytes"));
                }

                self.read_continuation = None;

                Ok(Some(recv_buf.to_vec()))
            }
            Err(e) => {

                if e.kind() == ErrorKind::WouldBlock {
                    println!("CONN : read encountered WouldBlock");

                    // We are being forced to try again, but we already read the two bytes off of the
                    // wire that determined the length. We need to store the message length so we can
                    // resume next time we get readable.
                    self.read_continuation = Some(msg_len as u64);
                    Ok(None)
                } else {
                    println!("Failed to read buffer for token {:?}, error: {}",
                             self.token,
                             e);
                    Err(e)
                }
            }
        }
    }

    fn read_message_length(&mut self) -> io::Result<Option<u64>> {
        if let Some(n) = self.read_continuation {
            return Ok(Some(n));
        }

        let mut buf = [0u8; 8];

        let bytes = match self.sock.read(&mut buf) {
            Ok(n) => n,
            Err(e) => {
                if e.kind() == ErrorKind::WouldBlock {
                    return Ok(None);
                } else {
                    return Err(e);
                }
            }
        };

        if bytes < 8 {
            println!("Found message length of {} bytes", bytes);
            return Err(Error::new(ErrorKind::InvalidData, "Invalid message length"));
        }

        let mut data: u64 = 0;
        unsafe {
            copy_nonoverlapping(buf.as_ptr(), &mut data as *mut u64 as *mut u8, 8);
        };
        let msg_len = data.to_be();
        //        let msg_len = BigEndian::read_u64(buf.as_ref());
        Ok(Some(msg_len))
    }

    pub fn writable(&mut self) -> io::Result<()> {

        try!(self.send_queue
            .pop()
            .ok_or(Error::new(ErrorKind::Other, "Could not pop send queue"))
            .and_then(|buf| {
                match self.write_message_length(&buf) {
                    Ok(None) => {
                        // put message back into the queue so we can try again
                        self.send_queue.push(buf);
                        return Ok(());
                    }
                    Ok(Some(())) => {
                        ()
                    }
                    Err(e) => {
                        println!("Failed to send buffer for {:?}, error: {}", self.token, e);
                        return Err(e);
                    }
                }

                match self.sock.write(&*buf) {
                    Ok(n) => {
                        println!("CONN : we wrote {} bytes", n);
                        self.write_continuation = false;
                        Ok(())
                    }
                    Err(e) => {
                        if e.kind() == ErrorKind::WouldBlock {
                            println!("client flushing buf; WouldBlock");

                            // put message back into the queue so we can try again
                            self.send_queue.push(buf);
                            self.write_continuation = true;
                            Ok(())
                        } else {
                            println!("Failed to send buffer for {:?}, error: {}", self.token, e);
                            Err(e)
                        }
                    }
                }
            }));

        if self.send_queue.is_empty() {
            self.interest.remove(Ready::writable());
        }

        Ok(())
    }

    fn write_message_length(&mut self, buf: &Rc<Vec<u8>>) -> io::Result<Option<()>> {
        if self.write_continuation {
            return Ok(Some(()));
        }

        let len = buf.len();
        let mut send_buf = [0u8; 8];

        unsafe {
            let bytes = transmute::<_, [u8; 8]>((len as u64).to_be());
            copy_nonoverlapping((&bytes).as_ptr(), send_buf.as_mut_ptr(), 8);
        }

        //        BigEndian::write_u64(&mut send_buf, len as u64);

        match self.sock.write(&send_buf) {
            Ok(n) => {
                println!("Sent message length of {} bytes", n);
                Ok(Some(()))
            }
            Err(e) => {
                if e.kind() == ErrorKind::WouldBlock {
                    println!("client flushing buf; WouldBlock");

                    Ok(None)
                } else {
                    println!("Failed to send buffer for {:?}, error: {}", self.token, e);
                    Err(e)
                }
            }
        }
    }

    pub fn send_message(&mut self, message: Rc<Vec<u8>>) -> io::Result<()> {
        println!("connection send_message; token={:?}", self.token);

        self.send_queue.push(message);

        if !self.interest.is_writable() {
            self.interest.insert(Ready::writable());
        }

        Ok(())
    }

    pub fn register(&mut self, poll: &mut Poll) -> io::Result<()> {
        println!("connection register; token={:?}", self.token);

        self.interest.insert(Ready::readable());

        poll.register(
            &self.sock,
            self.token,
            self.interest,
            PollOpt::edge() | PollOpt::oneshot()
        ).and_then(|(),| {
            self.is_idle = false;
            Ok(())
        }).or_else(|e| {
            println!("Failed to reregister {:?}, {:?}", self.token, e);
            Err(e)
        })
    }

    /// Re-register interest in read events with poll.
    pub fn reregister(&mut self, poll: &mut Poll) -> io::Result<()> {
        println!("connection reregister; token={:?}", self.token);

        poll.reregister(
            &self.sock,
            self.token,
            self.interest,
            PollOpt::edge() | PollOpt::oneshot()
        ).and_then(|(),| {
            self.is_idle = false;
            Ok(())
        }).or_else(|e| {
            println!("Failed to reregister {:?}, {:?}", self.token, e);
            Err(e)
        })
    }

    pub fn mark_reset(&mut self) {
        println!("connection mark_reset; token={:?}", self.token);

        self.is_reset = true;
    }

    #[inline]
    pub fn is_reset(&self) -> bool {
        self.is_reset
    }

    pub fn mark_idle(&mut self) {
        println!("connection mark_idle; token={:?}", self.token);

        self.is_idle = true;
    }

    #[inline]
    pub fn is_idle(&self) -> bool {
        self.is_idle
    }
}
