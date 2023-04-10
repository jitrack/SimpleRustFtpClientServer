use std::fs::File;
use std::io::{Read, Write};
use std::mem::size_of;
use std::net::TcpStream;
use std::slice;
use colored::*;

pub struct Tcp {
    pub stream: TcpStream,
}

// impl Tcp {
//     pub fn write<T>(&mut self, packet: TcpPacket<T>) -> std::io::Result<()> {
//         let raw_packet = unsafe { packet.serialize() };
//         println!("{}: {:?}", "Send:".truecolor(252, 152, 3).bold(), String::from_utf8_lossy(&raw_packet));
//         unsafe { self.stream.write(raw_packet)?; }
//         Ok(())
//     }
//
//     pub fn read<T: Default>(&mut self) -> std::io::Result<TcpPacket<T>> {
//         let num_bytes = size_of::<TcpPacket<T>>();
//         let mut buffer = vec![0u8; num_bytes];
//         self.stream.read_exact(&mut buffer)?;
//         let packet = TcpPacket::<T>::deserialize::<T>(&buffer)?;
//         println!("{}: {:?}", "Receive:".truecolor(252, 190, 3).bold(), String::from_utf8_lossy(&buffer));
//         return Ok(packet);
//     }
// }

const MESSAGE_SIZE: usize = 256;

pub fn read_message(stream: &mut TcpStream) -> Vec<u8> {
    let mut received: Vec<u8> = vec![];
    let mut rx_bytes = [0u8; MESSAGE_SIZE];
    loop {
        let bytes_read = stream.read(&mut rx_bytes).expect("Tcp: Could not read message");
        received.extend_from_slice(&rx_bytes[..bytes_read]);
        if bytes_read < MESSAGE_SIZE {
            break;
        }
    }
    // let mut line = String::from_utf8_lossy(received).unwrap();
    println!("{} {}: {:?}", "TCP Receive from".truecolor(252, 190, 3).bold(), stream.peer_addr().unwrap().to_string().underline().bold(), received);
    return received

}

pub fn write_message(stream: &mut TcpStream, data: Vec<u8>) {
    stream.write(data.as_slice()).expect("Something went wrong writing command");
    stream.flush().expect("Something went wrong flushing stream");
    println!("{} {}: {:?}", "TCP Send to".truecolor(252, 148, 3).bold(), stream.peer_addr().unwrap().to_string().underline().bold(), data);
}
