
// K-Ordered Sequence Generator for Distributed Systems by Twitter

extern crate num;
extern crate byteorder;

use std::time;
use self::num::BigUint;
use self::byteorder::{LittleEndian, WriteBytesExt};

#[derive(Debug)]
pub enum SequenceError {
    ClockIsRunningBackwards,
}

pub struct Sequence {
    identifier: [u8; 6],
    last_generated_time_ms: u64,
    counter: u16,
}

#[derive(PartialEq)]
pub enum Endianness {
    LittleEndian,
    BigEndian,
}

impl Sequence {
    pub fn new_from_identifier(identifier: Vec<u8>) -> Sequence {
        let mut a_identifier: [u8; 6] = [0 as u8; 6];
        a_identifier.clone_from_slice(&identifier);
        Sequence::new(a_identifier, Endianness::LittleEndian)
    }

    pub fn new(mut identifier: [u8; 6], endian: Endianness) -> Sequence {
        if identifier.len() < 6 {
            panic!("Identifier must have a length of 6");
        }

        if endian == Endianness::BigEndian {
            identifier.reverse();
        }

        Sequence {
            identifier: identifier,
            last_generated_time_ms: Sequence::current_time_in_ms(),
            counter: 0,
        }
    }

    fn current_time_in_ms() -> u64 {
        let now_ts = match time::SystemTime::now().duration_since(time::UNIX_EPOCH) {
            Ok(dur) => dur,
            Err(err) => err.duration(),
        };
        now_ts.as_secs() * 1000 + (now_ts.subsec_nanos() / 1000_000) as u64
    }

    /// Creates a new Sequence ID from the identifier, current time, and an internal counter.
    /// Identifiers are generated as 128-bit numbers:
    /// * 64-bit timestamp as milliseconds since the dawn of time (January 1, 1970)
    /// * 48-bit worker identifier
    /// * 16-bit sequence number that is incremented when more than one
    /// identifier is requested in the same millisecond and reset to 0 when the clock moves forward

    fn construct_id(&mut self) -> BigUint {
        let mut bytes = [0 as u8; 16];
        bytes[0] = self.counter as u8;
        bytes[1] = (self.counter >> 8) as u8;

        for (pos, byte) in self.identifier.iter().enumerate() {
            bytes[pos + 2] = *byte;
        }

        let mut wtr = vec![];

        wtr.write_u64::<LittleEndian>(self.last_generated_time_ms).unwrap();

        for (pos, w) in wtr.into_iter().enumerate() {
            bytes[pos + 8] = w;
        }

        BigUint::from_bytes_le(&bytes)
    }

    fn update(&mut self) -> Result<(), SequenceError> {
        let current_time_in_ms = Sequence::current_time_in_ms();
        if self.last_generated_time_ms > current_time_in_ms {
            return Result::Err(SequenceError::ClockIsRunningBackwards);
        }
        if self.last_generated_time_ms < current_time_in_ms {
            self.counter = 0;
        } else {
            self.counter += 1;
        }

        self.last_generated_time_ms = current_time_in_ms;

        Ok(())
    }

    pub fn get_id(&mut self) -> Result<BigUint, SequenceError> {
        self.update().map(|_| self.construct_id())
    }
}

#[test]
fn ids_change_over_time() {
    use std::time::Duration;
    use std::thread;
    let mut f1 = Sequence::new_from_identifier(vec![0, 1, 2, 3, 4, 5]);
    let id1 = f1.get_id().unwrap();
    thread::sleep(Duration::from_millis(50));
    let id2 = f1.get_id().unwrap();
    println!("{} < {}", id1, id2);
    assert!(id1 < id2);
}

#[test]
fn ids_change_quickly() {
    let mut f1 = Sequence::new([0, 1, 2, 3, 4, 5], Endianness::LittleEndian);
    let id3 = f1.get_id().unwrap();
    let id4 = f1.get_id().unwrap();
    println!("{} < {}", id3, id4);
    assert!(id3 < id4);
}
