
use std::ops::Deref;
use std::rc::Rc;
use std::cell::RefCell;
use std::io::{self, ErrorKind, Error};
use mio;
use std::fmt;
use network::message::Message;
use network::tcp::pipe;
use network::endpoint;
use network::endpoint::Context;
use network::tcp::pipe::*;

pub struct TestStepStreamSensor {
    sent_handshakes: Vec<(u16, u16)>,
    received_handshakes: usize,
    start_send_result: Option<bool>,
    resume_send_result: Option<bool>,
    start_recv_result: Option<Message>,
    resume_recv_result: Option<Message>,
}

impl TestStepStreamSensor {
    pub fn new() -> TestStepStreamSensor {
        TestStepStreamSensor {
            sent_handshakes: Vec::new(),
            received_handshakes: 0,
            start_send_result: Some(true),
            resume_send_result: None,
            start_recv_result: None,
            resume_recv_result: None,
        }
    }

    pub fn get_sent_handshakes(&self) -> &[(u16, u16)] {
        &self.sent_handshakes
    }

    fn push_sent_handshake(&mut self, sent_handshake: (u16, u16)) {
        self.sent_handshakes.push(sent_handshake);
    }

    pub fn get_received_handshakes(&self) -> usize {
        self.received_handshakes
    }

    fn push_received_handshake(&mut self) {
        self.received_handshakes += 1;
    }

    fn take_start_send_result(&mut self) -> Option<bool> {
        self.start_send_result.take()
    }

    pub fn set_start_send_result(&mut self, res: Option<bool>) {
        self.start_send_result = res;
    }

    fn take_resume_send_result(&mut self) -> Option<bool> {
        self.resume_send_result.take()
    }

    pub fn set_resume_send_result(&mut self, res: Option<bool>) {
        self.resume_send_result = res;
    }

    fn take_start_recv_result(&mut self) -> Option<Message> {
        self.start_recv_result.take()
    }

    pub fn set_start_recv_result(&mut self, res: Option<Message>) {
        self.start_recv_result = res;
    }

    fn take_resume_recv_result(&mut self) -> Option<Message> {
        self.resume_recv_result.take()
    }

    pub fn set_resume_recv_result(&mut self, res: Option<Message>) {
        self.resume_recv_result = res;
    }
}

pub struct TestStepStream {
    sensor: Rc<RefCell<TestStepStreamSensor>>,
    send_handshake_ok: bool,
    recv_handshake_ok: bool,
    pending_send: bool,
    pending_recv: bool,
}

impl TestStepStream {
    pub fn new() -> TestStepStream {
        let sensor = TestStepStreamSensor::new();
        TestStepStream::with_sensor(Rc::new(RefCell::new(sensor)))
    }
    pub fn with_sensor(sensor: Rc<RefCell<TestStepStreamSensor>>) -> TestStepStream {
        TestStepStream {
            sensor: sensor,
            send_handshake_ok: true,
            recv_handshake_ok: true,
            pending_send: false,
            pending_recv: false,
        }
    }
    pub fn set_send_handshake_ok(&mut self, send_handshake_ok: bool) {
        self.send_handshake_ok = send_handshake_ok;
    }
}

impl AsyncPipeStub for TestStepStream {}

impl mio::Evented for TestStepStream {
    fn register(&self,
                _: &mio::Poll,
                _: mio::Token,
                _: mio::Ready,
                _: mio::PollOpt)
                -> io::Result<()> {
        unimplemented!();
    }
    fn reregister(&self,
                  _: &mio::Poll,
                  _: mio::Token,
                  _: mio::Ready,
                  _: mio::PollOpt)
                  -> io::Result<()> {
        unimplemented!();
    }
    fn deregister(&self, _: &mio::Poll) -> io::Result<()> {
        unimplemented!();
    }
}

impl Deref for TestStepStream {
    type Target = mio::Evented;
    fn deref(&self) -> &Self::Target {
        self
    }
}

impl Handshake for TestStepStream {
    fn send_handshake(&mut self, pids: (u16, u16)) -> io::Result<()> {
        self.sensor.borrow_mut().push_sent_handshake(pids);
        if self.send_handshake_ok {
            Ok(())
        } else {
            Err(Error::new(ErrorKind::Other, "test"))
        }
    }
    fn recv_handshake(&mut self, _: (u16, u16)) -> io::Result<()> {
        self.sensor.borrow_mut().push_received_handshake();
        if self.recv_handshake_ok {
            Ok(())
        } else {
            Err(Error::new(ErrorKind::Other, "test"))
        }
    }
}

impl Sender for TestStepStream {
    fn start_send(&mut self, _: Rc<Message>) -> io::Result<bool> {
        match self.sensor.borrow_mut().take_start_send_result() {
            Some(true) => {
                self.pending_send = false;
                Ok(true)
            }
            Some(false) => {
                self.pending_send = true;
                Ok(false)
            }
            None => Err(Error::new(ErrorKind::Other, "test")),
        }
    }

    fn resume_send(&mut self) -> io::Result<bool> {
        match self.sensor.borrow_mut().take_resume_send_result() {
            Some(true) => {
                self.pending_send = false;
                Ok(true)
            }
            Some(false) => {
                self.pending_send = true;
                Ok(false)
            }
            None => Err(Error::new(ErrorKind::Other, "test")),
        }
    }

    fn has_pending_send(&self) -> bool {
        self.pending_send
    }
}

impl Receiver for TestStepStream {
    fn start_recv(&mut self) -> io::Result<Option<Message>> {
        match self.sensor.borrow_mut().take_start_recv_result() {
            Some(msg) => {
                self.pending_recv = false;
                Ok(Some(msg))
            }
            None => {
                self.pending_recv = true;
                Ok(None)
            }
        }
    }

    fn resume_recv(&mut self) -> io::Result<Option<Message>> {
        match self.sensor.borrow_mut().take_resume_recv_result() {
            Some(msg) => {
                self.pending_recv = false;
                Ok(Some(msg))
            }
            None => {
                self.pending_recv = true;
                Ok(None)
            }
        }
    }

    fn has_pending_recv(&self) -> bool {
        self.pending_recv
    }
}

pub struct TestPipeContext {
    registrations: Vec<(mio::Ready, mio::PollOpt)>,
    reregistrations: Vec<(mio::Ready, mio::PollOpt)>,
    deregistrations: usize,
    raised_events: Vec<pipe::Event>,
}

impl TestPipeContext {
    pub fn new() -> TestPipeContext {
        TestPipeContext {
            registrations: Vec::new(),
            reregistrations: Vec::new(),
            deregistrations: 0,
            raised_events: Vec::new(),
        }
    }
    pub fn get_registrations(&self) -> &[(mio::Ready, mio::PollOpt)] {
        &self.registrations
    }
    pub fn get_reregistrations(&self) -> &[(mio::Ready, mio::PollOpt)] {
        &self.reregistrations
    }
    pub fn get_deregistrations(&self) -> usize {
        self.deregistrations
    }
    pub fn get_raised_events(&self) -> &[pipe::Event] {
        &self.raised_events
    }
}

impl endpoint::EndpointRegistrar for TestPipeContext {
    fn register(&mut self, _: &mio::Evented, interest: mio::Ready, opt: mio::PollOpt) {
        self.registrations.push((interest, opt));
    }
    fn reregister(&mut self, _: &mio::Evented, interest: mio::Ready, opt: mio::PollOpt) {
        self.reregistrations.push((interest, opt));
    }
    fn deregister(&mut self, _: &mio::Evented) {
        self.deregistrations += 1;
    }
}

impl fmt::Debug for TestPipeContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TestPipeContext")
    }
}

impl Context for TestPipeContext {
    fn raise(&mut self, evt: pipe::Event) {
        self.raised_events.push(evt);
    }
}
