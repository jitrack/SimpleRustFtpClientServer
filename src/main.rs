extern crate exitcode;

use std::env;

mod client;
mod server;
mod core;
mod udp;
mod tcp;

use client::client::Client;
use server::server::Server;
use crate::core::CoreT;

pub fn print_exception<T>(val: bincode::Result<T>) -> T {
    return match val {
        Ok(_) => val.unwrap(),
        Err(e) => {
            println!("Error: {}", e);
            std::process::exit(exitcode::SOFTWARE);
        }
    }
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let mut core: Option<Box<dyn CoreT>> = match args[1].as_str() {
        "-c" => Some(Box::new(Client::new(String::from("localhost"), 22222)?)),
        "-s" => Some(Box::new(Server::new(String::from("0.0.0.0"), 22222)?)),
        _ => None,
    };
    if core.is_some() {
        core.unwrap().run()?;
    } else {
        println!("Invalid argument");
        std::process::exit(exitcode::USAGE);
    }
    Ok(())
}
