use std::net::UdpSocket;
use std::time::Duration;
use colored::*;

pub struct Udp {
    pub socket: UdpSocket
}

impl Udp {
    pub fn read<T>(&mut self) -> Option<T> where T: for<'a> serde::de::Deserialize<'a>, {
        let mut received: Vec<u8> = vec![];
        let mut rx_bytes = [0u8; 1300];
        let (bytes_read, peer_addr) = match self.socket.recv_from(&mut rx_bytes) {
            Ok((bytes_read, peer_addr)) => {
                println!("UDP: Received {} bytes from {}", bytes_read, peer_addr);
                (bytes_read, peer_addr)
            }
            Err(e) => {
                if self.socket.read_timeout().expect("Udp socket timeout").is_some() {
                    println!("UDP: Timeout");
                }
                return None;
            }
        };
        received.extend_from_slice(&rx_bytes[..bytes_read]);
        println!("{} {}: {:?}", "UDP Receive from".truecolor(252, 190, 3).bold(), peer_addr.to_string().underline().bold(), received);
        return Some(bincode::deserialize::<T>(&received[..]).unwrap());

    }

    pub fn read_raw(&mut self) -> Option<Vec<u8>> {
        let mut received: Vec<u8> = vec![];
        let mut rx_bytes = [0u8; 1300];
        let (bytes_read, peer_addr) = match self.socket.recv_from(&mut rx_bytes) {
            Ok((bytes_read, peer_addr)) => {
                println!("UDP: Received {} bytes from {}", bytes_read, peer_addr);
                (bytes_read, peer_addr)
            }
            Err(e) => {
                if self.socket.read_timeout().expect("Udp socket timeout").is_some() {
                    println!("UDP: Timeout");
                }
                return None;
            }
        };
        received.extend_from_slice(&rx_bytes[..bytes_read]);
        println!("{} {}: {:?}", "UDP Receive from".truecolor(252, 190, 3).bold(), peer_addr.to_string().underline().bold(), received);
        return Some(received);

    }

    pub fn write<T>(&mut self, data: & T) where T: serde::Serialize {
        let bytes = bincode::serialize(data).unwrap();
        self.socket.send(bytes.as_slice()).expect("couldn't send message");
        println!("{} {}: {:?}", "UDP Send to".truecolor(252, 148, 3).bold(), self.peer_addr_to_string().underline().bold(), bytes);
    }

    pub fn write_raw(&mut self, data: Vec<u8>) {
        self.socket.send(data.as_slice()).expect("couldn't send message");
        println!("{} {}: {:?}", "UDP Send to".truecolor(252, 148, 3).bold(), self.peer_addr_to_string().underline().bold(), data);
    }

    pub fn peer_addr_to_string(&self) -> String {
        return self.socket.peer_addr().unwrap().to_string();
    }

    pub fn local_addr_to_string(&self) -> String {
        return self.socket.local_addr().unwrap().to_string();
    }

    pub fn set_read_timeout(&self, dur: Option<Duration>) {
        self.socket.set_read_timeout(dur).expect("set_read_timeout call failed")
    }

    fn drop(&mut self) {
        println!("UDP: Dropping socket");
    }
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
