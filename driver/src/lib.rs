pub mod prelude;

use errors::Result;
use packets::{ClientCommand, Packet, PacketSender, ServerResponse};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct Client {
    stream: tokio::net::TcpStream,
}

impl Client {
    pub async fn connect(addr: &str) -> Result<Self> {
        let stream = tokio::net::TcpStream::connect(addr).await?;
        Ok(Self { stream })
    }

    pub fn into_ref(self) -> ClientRef {
        ClientRef::from(self)
    }
}

impl PacketSender<ClientCommand, ServerResponse> for &mut Client {
    async fn send(self, packet: &ClientCommand) -> Result<ServerResponse> {
        packet.write(&mut self.stream).await?;

        let response = ServerResponse::read(&mut self.stream).await?;

        if let ServerResponse::Error { err } = response {
            Err(err)
        } else {
            Ok(response)
        }
    }
}

#[derive(Clone)]
pub struct ClientRef {
    shared_client: Arc<RwLock<Client>>,
}

impl From<Client> for ClientRef {
    fn from(value: Client) -> Self {
        Self {
            shared_client: Arc::new(RwLock::new(value)),
        }
    }
}

impl ClientRef {
    pub async fn connect(addr: &str) -> Result<Self> {
        let client = Client::connect(addr).await?;
        Ok(Self {
            shared_client: Arc::new(RwLock::new(client)),
        })
    }
}

impl PacketSender<ClientCommand, ServerResponse> for &ClientRef {
    async fn send(self, packet: &ClientCommand) -> Result<ServerResponse> {
        self.shared_client.write().await.send(packet).await
    }
}
