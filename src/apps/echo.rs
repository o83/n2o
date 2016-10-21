extern crate kernel;

use std::env;
use std::net::SocketAddr;
use kernel::abstractions::futures::future::Future;
use kernel::abstractions::futures::{finished, lazy};
use kernel::abstractions::streams::stream::Stream;
use kernel::reactors::io_token::{copy, Io};
use kernel::reactors::sched::Core;
use kernel::streams::network::tcp::TcpListener;

fn main() {
    let addr = env::args().nth(1).unwrap_or("127.0.0.1:8080".to_string());
    let addr = addr.parse::<SocketAddr>().unwrap();
    let mut l = Core::new().unwrap();
    let handle = l.handle();
    let socket = TcpListener::bind(&addr, &handle).unwrap();
    println!("Listening on: {}", addr);
    let done = socket.incoming().for_each(move |(socket, addr)| {
        let msg = lazy::new(||finished::new(socket.split()))
                        .and_then(|(reader, writer)| copy(reader, writer))
                        .map(move |amt|{
            println!("wrote {} bytes to {}", amt, addr)
        }).map_err(|e| {
            panic!("error: {}", e);
        });
        handle.spawn(msg);
        Ok(())
    });
    l.run(done).unwrap();
}
