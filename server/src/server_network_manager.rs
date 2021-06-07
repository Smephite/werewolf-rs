use werewolf_rs::network_manager::NetworkManager;

pub struct ServerNetworkManager;

impl ServerNetworkManager {
    pub fn new() -> Self {
        Self
    }
    
}

impl NetworkManager for ServerNetworkManager {
    fn send_raw(&self, data: &str)
    {
        println!("Server send: {}", data);
        let _ = self.receive_raw(data);
    }



}