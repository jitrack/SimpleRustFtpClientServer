use num_derive::FromPrimitive;
use serde::{Deserialize, Serialize};

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

pub static ERROR_OCCURRED: &'static str = "Error: Failed to create file";
