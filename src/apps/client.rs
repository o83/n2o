
// TCP Client Sample

extern crate core;

use core::mem::transmute;
use core::ptr::copy_nonoverlapping;

use std::io::prelude::*;
use std::net::TcpStream;
use std::thread;

static NTHREADS: i32 = 10;

fn main() {

    for i in 0..NTHREADS {

        let _ = thread::spawn(move || {

            let mut stream = TcpStream::connect("127.0.0.1:8000").unwrap();

            loop {
                let msg = format!("the answer is {}", i);
                let mut buf = [0u8; 8];

                println!("thread {}: Sending 2 over message length of {}",
                         i,
                         msg.len());
                unsafe {
                    let bytes = transmute::<_, [u8; 8]>((msg.len() as u64).to_be());
                    copy_nonoverlapping((&bytes).as_ptr(), buf.as_mut_ptr(), 8);
                }

                stream.write_all(buf.as_ref()).unwrap();
                stream.write_all(msg.as_ref()).unwrap();

                let mut buf = [0u8; 8];
                stream.read(&mut buf).unwrap();

                let mut data: u64 = 0;
                unsafe {
                    copy_nonoverlapping(buf.as_ptr(), &mut data as *mut u64 as *mut u8, 8);
                };
                let msg_len = data.to_be();
                println!("thread {}: 2 Reading message length of {}", i, msg_len);

                let mut r = [0u8; 256];
                let s_ref = <TcpStream as Read>::by_ref(&mut stream);

                match s_ref.take(msg_len).read(&mut r) {
                    Ok(0) => {
                        println!("thread {}: 0 bytes read", i);
                    }
                    Ok(n) => {
                        println!("thread {}: {} bytes read", i, n);

                        let s = std::str::from_utf8(&r[..]).unwrap();
                        println!("thread {} read = {}", i, s);
                    }
                    Err(e) => {
                        return;
                        //                        panic!("thread {}: {}", i, e);
                    }
                }
            }
        });
    }

    loop {}
}
