mod connection;
mod container;
pub(crate) mod data;
mod state;

use crate::state::State;
use errors::Result;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let stream = tokio::net::TcpListener::bind("127.0.0.1:3599").await?;

    let state = State::default();

    log::info!("...Accepting connections...");

    loop {
        let state = state.clone();

        let (stream, addr) = stream.accept().await?;

        log::debug!("New connection from {}", addr);
        connection::handle(state, stream, addr);
    }
}
