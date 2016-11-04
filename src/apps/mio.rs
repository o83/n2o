
// MIO Compatibility Sample

extern crate kernel;

use kernel::io::*;
use kernel::io::poll::*;
use kernel::io::ready::*;
use kernel::io::token::*;
use kernel::io::options::*;
use kernel::io::tcp::*;

const SERVER: Token = Token(0);
const CLIENT: Token = Token(1);


fn main() {

    let addr = "127.0.0.1:13265".parse().unwrap();
    let server = TcpListener::bind(&addr).unwrap();
    let sock = TcpStream::connect(&addr).unwrap();
    let mut poll = Poll::new().unwrap();
    let mut events = Events::with_capacity(1024);
    poll.register(&server, SERVER, Ready::readable(), PollOpt::edge()).unwrap();
    poll.register(&sock, CLIENT, Ready::readable(), PollOpt::edge()).unwrap();
    loop {
        poll.poll(&mut events, None).unwrap();
        for event in events.iter() {
            match event.token() {
                  SERVER => { let _ = server.accept(); }
                  CLIENT => { return; }
                       _ => unreachable!(),
            }
        }
    }
}
