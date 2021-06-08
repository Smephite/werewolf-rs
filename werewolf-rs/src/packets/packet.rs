use anyhow::Result;
use serde::{Deserialize, Serialize};

pub fn serialize_packet<P: Serialize>(packet: &P) -> Result<String> {
    let raw: String = serde_json::ser::to_string(packet)?;
    Ok(raw)
}

pub fn deserialize_packet<'a, P: Deserialize<'a>>(raw: &'a str) -> Result<P> {
    let packet: P = serde_json::from_str(raw)?;
    Ok(packet)
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ToServer {
    Pong(String),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ToClient {
    Ping(String),
}
