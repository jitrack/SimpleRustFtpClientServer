use serde::{Deserialize, Serialize};
use crate::core::{CommandId, FtpStatusCode};
use serde_with::{serde_as, Bytes};

// TCP
#[serde_as]
#[derive(Serialize, Deserialize, Debug)]
pub struct PutPacket {
    pub cmd: CommandId,
    pub size: u64,
    #[serde_as(as = "Bytes")]
    pub name: [u8; 40],
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug)]
pub struct ResponsePacket {
    pub status: FtpStatusCode,
    #[serde_as(as = "Bytes")]
    pub message: [u8; 150],
}


// UDP
#[serde_as]
#[derive(Serialize, Deserialize, Debug)]
pub struct FilePacket {
    pub index: u64,
    pub is_last: bool,
    pub data_size: usize,
    #[serde_as(as = "Bytes")]
    pub data: [u8; 1024],
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug)]
pub struct ResponseFilePacket {
    pub status: FtpStatusCode,
    pub index: u64,
}
