use serde_json::Result;
use crate::packets::packet::Packet;

pub trait NetworkManager {
    fn receive_raw(&self, input: &str) -> Result<()>;
    fn send_packet(&self, packet: &Packet) -> Result<()>;
    fn send_raw(&self, string: &str);
}

default impl<T> NetworkManager for T{
    fn receive_raw(&self, input: &str) -> Result<()>{
        println!("Received raw {}", input);
        let packet = serde_json::from_str::<Packet>(input)?;
        println!("Received packet {:?}", packet);
        Ok(())
    }
    fn send_packet(&self, packet: &Packet) -> Result<()>{
        println!("Sending packet {:?}", packet);
        self.send_raw(&serde_json::to_string(packet)?);
        Ok(())
    }
}