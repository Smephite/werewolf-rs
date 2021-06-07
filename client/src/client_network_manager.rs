use werewolf_rs::network_manager::NetworkManager;

pub struct ClientNetworkManager;

impl ClientNetworkManager {
    pub fn new() -> Self {
        Self
    }
    
}

impl NetworkManager for ClientNetworkManager {
    fn send_raw(&self, data: &str)
    {
        println!("Client send: {}", data);
        let _ = self.receive_raw(data);
    }



}