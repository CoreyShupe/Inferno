use errors::Result;
pub use packets::value::ValueType;
pub use packets::{ClientCommand, Packet, ServerResponse};
use std::future::Future;
use std::sync::Arc;
use tokio::sync::RwLock;

macro_rules! async_trait_command {
    ($(fn $name:ident($($field:ident: $field_type:ty),*) {
        $create_response:expr
    })*) => {
        $(#[allow(async_fn_in_trait)]
        async fn $name(self, $($field: $field_type),*) -> Result<ServerResponse> {
            self.send(&$create_response).await
        })*
    }
}

pub trait CommandSender: Sized {
    fn send(self, packet: &ClientCommand) -> impl Future<Output = Result<ServerResponse>>;

    async_trait_command! {
        fn get(key: &str) {
            ClientCommand::Get { key: key.to_string() }
        }

        fn set(key: &str, value: ValueType) {
            ClientCommand::Set { key: key.to_string(), value }
        }

        fn del(key: &str) {
            ClientCommand::Del { key: key.to_string() }
        }
    }
}

pub struct Client {
    stream: tokio::net::TcpStream,
}

impl Client {
    pub async fn connect(addr: &str) -> errors::Result<Self> {
        let stream = tokio::net::TcpStream::connect(addr).await?;
        Ok(Self { stream })
    }
}

impl CommandSender for &mut Client {
    async fn send(self, packet: &ClientCommand) -> errors::Result<ServerResponse> {
        packet.write(&mut self.stream).await?;
        Ok(ServerResponse::read(&mut self.stream).await?)
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
    pub async fn connect(addr: &str) -> errors::Result<Self> {
        let client = Client::connect(addr).await?;
        Ok(Self {
            shared_client: Arc::new(RwLock::new(client)),
        })
    }
}

impl CommandSender for &ClientRef {
    async fn send(self, packet: &ClientCommand) -> errors::Result<ServerResponse> {
        self.shared_client.write().await.send(packet).await
    }
}
