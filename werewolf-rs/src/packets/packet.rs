#[derive(Serialize, Deserialize, Debug)]
pub enum Packet<'a> {
    #[serde(borrow)]
    ToServer(ToServer<'a>),
    ToClient(ToClient<'a>),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ToServer<'a> {
    Pong(&'a str),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ToClient<'a> {
    Ping(&'a str),
}
