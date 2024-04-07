use crate::state::State;
use packets::value::ValueType;
use packets::{ClientCommand, Packet, ServerResponse};
use std::net::SocketAddr;
use tokio::net::TcpStream;
use tokio::task::JoinHandle;

pub fn handle(
    state: State,
    stream: TcpStream,
    _addr: SocketAddr,
) -> JoinHandle<errors::Result<()>> {
    tokio::spawn(async move { inner_handle(state, stream).await })
}

async fn inner_handle(state: State, stream: TcpStream) -> errors::Result<()> {
    let (mut read, mut write) = stream.into_split();
    loop {
        let command = ClientCommand::read(&mut read).await?;

        log::info!("Command: {:?}", command);

        let response = match command {
            ClientCommand::Set { key, value } => {
                state.set(&key, value).await;
                ServerResponse::Ok
            }
            ClientCommand::Get { key } => {
                let value = state.get(&key).await.unwrap_or(ValueType::None);
                log::info!("Value: {:?}", value);
                ServerResponse::OkSingle { value }
            }
            ClientCommand::Del { key } => {
                let value = state.del(&key).await.unwrap_or(ValueType::None);
                log::info!("Value: {:?}", value);
                ServerResponse::OkSingle { value }
            }
        };

        response.write(&mut write).await?;
    }
}
