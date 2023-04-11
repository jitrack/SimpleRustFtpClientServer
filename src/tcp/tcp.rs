use std::fs::File;
use std::io::{Read, Write};
use std::mem::size_of;
use std::net::TcpStream;
use std::slice;
use colored::*;

pub struct Tcp {
    pub stream: TcpStream,
}

impl Tcp {
    pub fn write<T>(&mut self, data: & T) where T: serde::Serialize {
        let bytes = bincode::serialize(data).unwrap();
        self.stream.write(bytes.as_slice()).expect("Something went wrong writing command");
        self.stream.flush().expect("Something went wrong flushing stream");
        println!("{} {}: {:?}", "TCP Send to".truecolor(252, 148, 3).bold(), self.peer_addr_to_string().underline().bold(), bytes);
    }

    pub fn read<T>(&mut self) -> T where T: for<'a> serde::de::Deserialize<'a>, {
        return bincode::deserialize::<T>(&self.read_raw(size_of::<T>())[..]).unwrap();
    }

    pub fn read_raw(&mut self, size: usize) -> Vec<u8> {
        let mut received: Vec<u8> = vec![];
        let mut rx_bytes = [0; 250];
        loop {
            let bytes_read = match self.stream.read(&mut rx_bytes) {
                Ok(bytes_read) => bytes_read,
                Err(e) => {
                    println!("TCP: Error: {}", e);
                    0
                }
            };



            // let bytes_read = self.stream.read(&mut rx_bytes).expect("Tcp: Could not read message");
            received.extend_from_slice(&rx_bytes[..bytes_read]);
            if bytes_read < MESSAGE_SIZE {
                break;
            }
        }
        // let mut line = String::from_utf8_lossy(received).unwrap();
        println!("{} {}: {:?}", "TCP Receive from".truecolor(252, 190, 3).bold(), self.peer_addr_to_string().underline().bold(), received);
        return received
    }

    pub fn peer_addr_to_string(&self) -> String {
        return self.stream.peer_addr().unwrap().to_string();
    }

    pub fn local_addr_to_string(&self) -> String {
        return self.stream.local_addr().unwrap().to_string();
    }

    fn drop(&mut self) {
        self.stream.shutdown(std::net::Shutdown::Both).expect("Could not shutdown stream");
    }
}

const MESSAGE_SIZE: usize = 256;
