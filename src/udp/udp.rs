use std::net::UdpSocket;
use colored::*;

pub struct Udp {

}

impl Udp {
    pub fn new() -> Self { Udp{} }
}

pub const FILE_BLOC_SIZE: usize = 1024;

pub fn read_message_udp(socket: &mut UdpSocket) -> Option<Vec<u8>> {
    let mut received: Vec<u8> = vec![];
    let mut rx_bytes = [0u8; 1300];
    let (bytes_read, peer_addr) = match socket.recv_from(&mut rx_bytes) {
        Ok((bytes_read, peer_addr)) => {
            println!("UDP: Received {} bytes from {}", bytes_read, peer_addr);
            (bytes_read, peer_addr)
        }
        Err(e) => {
            if socket.read_timeout().expect("Udp socket timeout").is_some() {
                println!("UDP: Timeout");
            }
            return None;
        }
    };
    received.extend_from_slice(&rx_bytes[..bytes_read]);
    println!("{} {}: {:?}", "UDP Receive from".truecolor(252, 190, 3).bold(), peer_addr.to_string().underline().bold(), received);
    return Some(received);

}

pub fn write_message_udp(socket: &mut UdpSocket, data: Vec<u8>) {
    socket.send(data.as_slice()).expect("couldn't send message");
    println!("{} {}: {:?}", "UDP Send to".truecolor(252, 148, 3).bold(), socket.peer_addr().unwrap().to_string().underline().bold(), data);
}
