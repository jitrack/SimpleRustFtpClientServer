use std::{env, thread, time};
use std::fs::File;
use crate::core::{CommandId, CoreT, FtpStatusCode};
use std::io::{self, BufRead, BufReader};
use std::net::{Ipv4Addr, Shutdown, SocketAddrV4, TcpStream, UdpSocket};
use std::io::{Read, Write};
use std::ops::Add;
use std::path::Path;
use std::process::Command;
use std::str::from_utf8;
use crate::tcp::tcp::{read_message, Tcp, write_message};
use colored::*;
use crate::tcp::packet::{FilePacket, PutPacket, ResponseFilePacket, ResponsePacket};
use std::os::unix::prelude::FileExt;
use num_traits::ToPrimitive;
use crate::print_exception;
use crate::udp::udp::{FILE_BLOC_SIZE, read_message_udp, write_message_udp};

pub struct Client {
    is_running: bool,
    // tcp.rs: Tcp,
    udp: UdpSocket,
    stream: TcpStream
}

trait ClientT {}

impl CoreT for Client {
    fn run(&mut self) -> std::io::Result<()> {
        loop {
            let (cmd, args) = self.get_commands();
            match cmd.to_lowercase().as_ref() {
                "exit" => {
                    println!("Closing connection");
                    break;
                }
                "get" => {
                    // self.get(&mut client, &args, ftp_mode, ftp_type)
                }
                "put" => {
                    self.put(&args);
                }
                // "help" | "?" | "usage" => utils::print_help(&args),
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
                println!("{} {}", "Host address:".bold(), format!("{}:{}", stream.local_addr().unwrap().ip(), stream.local_addr().unwrap().port()).underline());
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
        let client = Client { is_running: true, udp, stream };
        Ok(client)
    }

    fn put(&mut self, file_path: &str) -> std::io::Result<()> {
        self.udp.set_read_timeout(Some(time::Duration::from_secs(3))).expect("set_read_timeout call failed");
        let mut file = File::open(file_path)?;
        let mut contents = String::new();
        let path = Path::new(file_path);
        let file_name = format!("{}", path.file_name().unwrap().to_str().unwrap());
        println!("file: \"{}\"", file_name);

        let mut packet = PutPacket { cmd: CommandId::Put, size: path.metadata()?.len(), name: [0; 40] };
        packet.name[..file_name.len()].copy_from_slice(file_name.as_bytes());

        write_message(&mut self.stream, bincode::serialize(&packet).unwrap());

        let received = read_message(&mut self.stream);
        let res = print_exception(bincode::deserialize::<ResponsePacket>(&received[..]));
        if res.status == FtpStatusCode::Error {
            println!("{} {}", "Error:".red(), String::from_utf8_lossy(&res.message));
            return Ok(());
        }

        let mut buffer: [u8; FILE_BLOC_SIZE] = [0; FILE_BLOC_SIZE];
        let mut offset = 0;
        let mut file_packet = FilePacket { index: 0, is_last: false, data_size: FILE_BLOC_SIZE, data: [0; FILE_BLOC_SIZE] };

        loop {
            let num_bytes_read = file.read_at(& mut buffer, offset)?;
            offset += num_bytes_read.to_u64().unwrap();

            file_packet.data = [0; FILE_BLOC_SIZE];
            if num_bytes_read != FILE_BLOC_SIZE {
                file_packet.is_last = true;
                file_packet.data_size = num_bytes_read;
            }
            file_packet.data[..buffer.len()].copy_from_slice(&buffer);
            let serialized_file_packet = bincode::serialize(&file_packet).unwrap();

            for retry in 1..7 {
                write_message_udp(&mut self.udp, serialized_file_packet.clone());
                match read_message_udp(&mut self.udp) {
                    Some(raw_packet) => {
                        // let response_file_packet = bincode::deserialize::<ResponseFilePacket>(&raw_packet[..]).unwrap();
                        // if response_file_packet.index == file_packet.index {
                        //     break;
                        // }
                        break;
                    },
                    None => {
                        if retry == 6 {
                            println!("{} {}", "Error:".red(), "Abort");
                            return Ok(())
                            // write_message_udp(udp, bincode::serialize(&ResponseFilePacket { index: file_packet.index, status: FtpStatusCode::Error }).unwrap());
                        }
                        println!("{} {} {} {}", "Error:".red(), "Server is not responding: ", retry, " try");
                    }
                }

            }

            file_packet.index += 1;

            if num_bytes_read != FILE_BLOC_SIZE {
                break;
            }
        }
        Ok(())
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
        println!("command: {}, args: {}", cmd, args);
        (s1, s2)
    }
}
