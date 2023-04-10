use std::fmt::format;
use std::io::Error;
use crate::core::{CommandId, CoreT, ERROR_OCCURRED, FtpStatusCode};
use std::net::{Shutdown, TcpListener, TcpStream, UdpSocket};
use std::{thread, time};
use std::io::{Read, Write};
use crate::tcp::tcp::{read_message, Tcp, write_message};
use colored::*;
use crate::tcp::packet::{FilePacket, PutPacket, ResponseFilePacket, ResponsePacket};
use crate::udp::udp::{read_message_udp, write_message_udp};
use std::str;

pub struct Server {
    is_running: bool,
    listener: TcpListener
}

trait ServerT {}

impl CoreT for Server {
    fn run(&mut self) -> std::io::Result<()> {
        for stream in self.listener.incoming() {
            match stream {
                Ok(stream) => {
                    println!("{} {}", "New connection: ".bold(), stream.peer_addr().unwrap().to_string().underline());
                    thread::spawn(move|| -> std::io::Result<()> {
                        handle_client(Tcp { stream });
                        Ok(())
                    });
                }
                Err(e) => {
                    println!("Error: {}", e);
                }
            }
        }
        Ok(())
    }
}

impl ServerT for Server {}

impl Server {
    pub fn new(ip: String, port: u16) -> std::io::Result<Self> {
        let address = format!("{}:{}", ip, port);
        let listener = TcpListener::bind(address.clone())?;
        println!("{} {address}", "Server is running and listen at address".green().bold());
        Ok(Server { is_running: true, listener })
    }
}

fn handle_client(mut tcp: Tcp) -> std::io::Result<()> {
    let mut udp = UdpSocket::bind(tcp.stream.local_addr().unwrap()).expect("Could not bind an UDP socket");
    udp.connect(tcp.stream.peer_addr().unwrap()).expect("Could not connect to server");
    loop {
        println!("{} {}", format!("{}", tcp.stream.peer_addr().unwrap().to_string().underline()).bold(), "wait for command".truecolor(252, 190, 3).bold());
        let packet = read_message(&mut tcp.stream);
        match num::FromPrimitive::from_u8(packet[0]) {
            Some(CommandId::Put) => {
                put(&bincode::deserialize::<PutPacket>(&packet[..]).unwrap(), &mut udp, & mut tcp.stream);
            }
            Some(_) => {

            }
            None => {
                println!("{} {}", "Unknown command from".red(), tcp.stream.peer_addr().unwrap().to_string().underline());
            }
        }
    }
}

fn put(packet: &PutPacket, udp: &mut UdpSocket, tcp: & mut TcpStream) -> std::io::Result<()> {
    let file_name = String::from_utf8_lossy(&packet.name).clone().to_string();
    let pwd = std::fs::canonicalize("files/")?;
    let path = format!("{}/{}", pwd.to_str().unwrap(), file_name.trim_matches(char::from(0))).to_string();
    let mut new_file = match std::fs::File::create(path) {
        Ok(file) => Ok(file),
        Err(e) => {
            let mut error_packet = ResponsePacket { status: FtpStatusCode::Error, message: [0; 150] };
            error_packet.message[..ERROR_OCCURRED.len()].copy_from_slice(ERROR_OCCURRED.as_bytes());
            write_message(tcp, bincode::serialize(&error_packet).unwrap());
            Err(())
        }
    }.expect("Error: Could not create file");
    write_message(tcp, bincode::serialize(&ResponsePacket{ status: FtpStatusCode::Ok, message: [0; 150] }).unwrap());
    loop {
        let raw_packet = read_message_udp(udp).unwrap();
        let file_packet = bincode::deserialize::<FilePacket>(&raw_packet[..]).unwrap();

        write_message_udp(udp, bincode::serialize(&ResponseFilePacket { index: file_packet.index, status: FtpStatusCode::Ok }).unwrap());

        new_file.write(&file_packet.data[..file_packet.data_size]);
        if file_packet.is_last {
            break;
        }
    }
    Ok(())
}