use std::fmt::format;
use std::io::Error;
use crate::core::{CommandId, CoreT, ERROR_FAILED_TO_CREATE_FILE, FtpStatusCode, receive_file, send_file};
use std::net::{Shutdown, TcpListener, TcpStream, UdpSocket};
use std::{thread, time};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use crate::tcp::tcp::{read_message, Tcp, write_message};
use colored::*;
use crate::tcp::packet::{CommandPacket, deserialize, FilePacket, FileInfoPacket, ResponseFilePacket, ResponsePacket};
use crate::udp::udp::{read_message_udp, Udp, write_message_udp};
use std::str;

pub struct Server {
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
        Ok(Server { listener })
    }
}

fn handle_client(mut tcp: Tcp) -> std::io::Result<()> {
    let mut socket = UdpSocket::bind(tcp.stream.local_addr().unwrap()).expect("Could not bind an UDP socket");
    socket.connect(tcp.peer_addr_to_string()).expect("Could not connect to server");
    let mut udp = Udp { socket };
    loop {
        println!("{} {}", format!("{}", tcp.peer_addr_to_string().underline()).bold(), "wait for command".truecolor(252, 190, 3).bold());
        let command = tcp.read::<CommandPacket>();
        match command.cmd {
            CommandId::Put => {
                put(&mut udp, & mut tcp);
            }
            CommandId::Get => {
                get(&mut udp, & mut tcp);
            }
            CommandId::Exit => {
                exit(&mut udp, & mut tcp);
                return Ok(());
            }
            _ => {
                println!("{} {}", "Unknown command from".red(), tcp.peer_addr_to_string().underline());
            }
        }
    }
}

fn put(udp: &mut Udp, tcp: &mut Tcp) -> std::io::Result<()> {
    let packet = tcp.read::<FileInfoPacket>();
    let path = get_path_from_filename(&packet.name)?;

    let mut file = match std::fs::File::create(path) {
        Ok(file) => file,
        Err(e) => {
            let mut error_packet = ResponsePacket { status: FtpStatusCode::Error, message: [0; 150] };
            error_packet.message[..ERROR_FAILED_TO_CREATE_FILE.len()].copy_from_slice(ERROR_FAILED_TO_CREATE_FILE.as_bytes());
            tcp.write(&error_packet);
            return Ok(())
        }
    };
    tcp.write(&ResponsePacket{ status: FtpStatusCode::Ok, message: [0; 150] });

    return receive_file(&mut file, udp);
}

fn get(udp: &mut Udp, tcp: &mut Tcp) -> std::io::Result<()> {
    let packet = tcp.read::<FileInfoPacket>();
    let path_string = get_path_from_filename(&packet.name)?;
    let path = Path::new(path_string.as_str());
    let mut file = match File::open(path_string) {
        Ok(file) => file,
        Err(_) => {
            let mut error_packet = ResponsePacket { status: FtpStatusCode::Error, message: [0; 150] };
            let error_message = "This file does not exist".to_string();
            error_packet.message[..error_message.len()].copy_from_slice(error_message.as_bytes());
            tcp.write(&error_packet);
            return Ok(())
        }
    };
    tcp.write(&ResponsePacket { status: FtpStatusCode::Ok, message: [0; 150] });

    let res_packet = tcp.read::<ResponsePacket>();
    if res_packet.status == FtpStatusCode::Error {
        println!("{} {}", "Error:".red(), String::from_utf8_lossy(&res_packet.message));
        return Ok(());
    }

    return send_file(&mut file, udp);
}

fn exit(udp: &mut Udp, tcp: &mut Tcp) -> std::io::Result<()> {
    println!("Connection with {} has been closed", format!("{}", tcp.peer_addr_to_string().underline()).bold());
    Ok(())
}


fn get_path_from_filename(filename: &[u8]) -> std::io::Result<String> {
    let filename_ = String::from_utf8_lossy(filename).clone().to_string();
    let pwd = std::fs::canonicalize("files/")?;
    return Ok(format!("{}/{}", pwd.to_str().unwrap(), filename_.trim_matches(char::from(0))).to_string());
}