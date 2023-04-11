use std::fs::File;
use std::path::Path;
use std::time;
use colored::Colorize;
use num_derive::FromPrimitive;
use serde::{Deserialize, Serialize};
use crate::tcp::packet::{FilePacket, FileInfoPacket, ResponseFilePacket, ResponsePacket};
use crate::tcp::tcp::Tcp;
use crate::udp::udp::{Udp};
use std::io::{Read, Write};
use std::os::unix::prelude::FileExt;
use num_traits::ToPrimitive;

pub trait CoreT {
    fn run(&mut self) -> std::io::Result<()>;
}

#[derive(Serialize, Deserialize, Debug, FromPrimitive)]
pub enum CommandId {
    Exit,
    Get,
    Put,
}

#[derive(Serialize, Deserialize, Debug, FromPrimitive, Eq, PartialEq)]
pub enum FtpStatusCode {
    Ok,
    Error,
}

pub const FILE_BLOC_SIZE: usize = 1024;

pub fn send_file(file: &mut File, udp: &mut Udp) -> std::io::Result<()> {
    udp.set_read_timeout(Some(time::Duration::from_secs(3)));

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
            udp.write_raw(serialized_file_packet.clone());
            match udp.read::<ResponseFilePacket>() {
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
                        // write_message_udp(udp, bincode::serialize(&ResponseFilePacket { index: file_packet.index, status: FtpStatusCode::Error }).unwrap());
                        return Ok(())
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

pub fn receive_file(file: &mut File, udp: &mut Udp) -> std::io::Result<()> {
    udp.set_read_timeout(None);
    loop {
        let packet = udp.read::<FilePacket>().unwrap();
        udp.write(&ResponseFilePacket { index: packet.index, status: FtpStatusCode::Ok });
        file.write(&packet.data[..packet.data_size]);
        if packet.is_last {
            break;
        }
    }
    Ok(())
}

pub static ERROR_FAILED_TO_CREATE_FILE: &'static str = "Failed to create file";
pub static ERROR_FILE_DOESNT_EXIST: &'static str = "File doesn't exist";
pub static ERROR_INVALID_NUMBER_OF_ARGUMENTS: &'static str = "Invalid number of arguments";
