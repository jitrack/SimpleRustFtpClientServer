use std::{env, thread, time};
use std::fs::File;
use crate::core::{CommandId, CoreT, ERROR_FAILED_TO_CREATE_FILE, ERROR_FILE_DOESNT_EXIST, ERROR_INVALID_NUMBER_OF_ARGUMENTS, FtpStatusCode, receive_file, send_file};
use std::io::{self, BufRead, BufReader};
use std::net::{Ipv4Addr, Shutdown, SocketAddrV4, TcpStream, UdpSocket};
use std::io::{Read, Write};
use std::ops::Add;
use std::path::Path;
use std::process::Command;
use std::str::from_utf8;
use crate::tcp::tcp::{Tcp};
use colored::*;
use crate::tcp::packet::{CommandPacket, FilePacket, FileInfoPacket, ResponseFilePacket, ResponsePacket};
use std::os::unix::prelude::FileExt;
use exitcode::OK;
use num_traits::ToPrimitive;
use crate::print_exception;
use crate::udp::udp::{Udp};

pub struct Client {
    tcp: Tcp,
    udp: Udp
}

trait ClientT {}

impl CoreT for Client {
    fn run(&mut self) -> std::io::Result<()> {
        loop {
            let (cmd, args) = self.get_commands();
            match cmd.to_lowercase().as_ref() {
                "exit" => {
                    self.exit();
                    println!("Closing connection");
                    break;
                }
                "get" => {
                    self.get(&args);
                }
                "put" => {
                    self.put(&args);
                }
                _ => {
                    println!("{} {}", "Unknown command:".red(), cmd);
                    continue;
                }
            }
        }
        Ok(())
    }
}

impl ClientT for Client {}

impl Client {
    pub fn new(ip: String, port: u16) -> std::io::Result<Self> {
        let address = format!("{}:{}", ip, port);
        let stream = match TcpStream::connect(&address) {
            Ok(mut stream) => {
                println!("{} {}", "Host address:".bold(), format!("{}", stream.local_addr().unwrap().to_string()).underline());
                println!("{} {}", "Successfully connected to server".green().bold(), address.underline());
                Ok(stream)
            },
            Err(e) => {
                println!("{} {}", "Failed to connect:".red(), e);
                Err(e)
            }
        }?;
        let udp = UdpSocket::bind(stream.local_addr().unwrap()).expect("Could not bind client socket");
        udp.connect(stream.peer_addr().unwrap()).expect("Could not connect to server");
        let client = Client { udp: Udp { socket: udp }, tcp: Tcp { stream } };
        Ok(client)
    }

    fn put(&mut self, file_path: &str) -> std::io::Result<()> {
        let args: Vec<&str> = file_path.split_whitespace().collect();
        if args.len() != 1 {
            println!("{} {}", "Error:".red(), ERROR_INVALID_NUMBER_OF_ARGUMENTS);
            return Ok(());
        }

        let path = Path::new(file_path);
        if !path.exists()  || !path.is_file() {
            println!("{} {}", "Error:".red(), ERROR_FILE_DOESNT_EXIST);
            return Ok(());
        }

        let file_name = format!("{}", path.file_name().unwrap().to_str().unwrap());

        self.tcp.write(&CommandPacket::new(CommandId::Put));

        let mut packet = FileInfoPacket { size: path.metadata()?.len(), name: [0; 40] };
        packet.name[..file_name.len()].copy_from_slice(file_name.as_bytes());
        self.tcp.write(&packet);

        let res = self.tcp.read::<ResponsePacket>();
        if res.status == FtpStatusCode::Error {
            println!("{} {}", "Error:".red(), String::from_utf8_lossy(&res.message));
            return Ok(());
        }

        let mut file = match File::open(file_path) {
            Ok(file) => file,
            Err(_) => {
                let mut error_packet = ResponsePacket { status: FtpStatusCode::Error, message: [0; 150] };
                let error_message = "This file does not exist".to_string();
                error_packet.message[..ERROR_FAILED_TO_CREATE_FILE.len()].copy_from_slice(ERROR_FAILED_TO_CREATE_FILE.as_bytes());
                self.tcp.write(&error_packet);
                return Ok(())
            }
        };
        self.tcp.write(&ResponsePacket { status: FtpStatusCode::Ok, message: [0; 150] });

        return send_file(&mut file, &mut self.udp);
    }

    fn get(&mut self, input: &str) -> std::io::Result<()> {
        let args: Vec<&str> = input.split_whitespace().collect();
        if args.len() < 1 || args.len() > 2 {
            println!("{} {}", "Error:".red(), ERROR_INVALID_NUMBER_OF_ARGUMENTS);
            return Ok(());
        }

        self.tcp.write(&CommandPacket::new(CommandId::Get));

        let mut packet = FileInfoPacket { size: 0, name: [0; 40] };
        packet.name[..args[0].len()].copy_from_slice(args[0].as_bytes());
        self.tcp.write(&packet);

        let res = self.tcp.read::<ResponsePacket>();
        if res.status == FtpStatusCode::Error {
            println!("{} {}", "Error:".red(), String::from_utf8_lossy(&res.message));
            return Ok(());
        }

        let mut locations = std::fs::canonicalize("./").unwrap().to_str().unwrap().to_string();
        if args.len() == 2 {
            locations = format!("{}/{}", locations, args[1]);
        } else {
            locations = format!("{}/{}", locations, args[0]);
        }


        let mut file = match std::fs::File::create(locations) {
            Ok(file) => file,
            Err(e) => {
                let mut error_packet = ResponsePacket { status: FtpStatusCode::Error, message: [0; 150] };
                error_packet.message[..ERROR_FAILED_TO_CREATE_FILE.len()].copy_from_slice(ERROR_FAILED_TO_CREATE_FILE.as_bytes());
                self.tcp.write(&error_packet);
                return Ok(())
            }
        };
        self.tcp.write(&ResponsePacket{ status: FtpStatusCode::Ok, message: [0; 150] });
        return receive_file(&mut file,&mut self.udp);
    }

    fn exit(&mut self) {
        self.tcp.write(&CommandPacket::new(CommandId::Exit));
    }

    fn get_commands(&self) -> (String, String) {
        print!("ftp> ");
        io::stdout().flush().unwrap();
        let mut buf = String::new();
        io::stdin().read_line(&mut buf).unwrap();
        let line = buf.trim();
        let (cmd, args) = match line.find(' ') {
            Some(pos) => (&line[0..pos], &line[pos + 1..]),
            None => (line, "".as_ref()),
        };
        let s1 = format!("{}", cmd);
        let s2 = format!("{}", args);
        (s1, s2)
    }
}
