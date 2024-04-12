use inferno::driver::prelude::*;

#[tokio::main]
async fn main() -> inferno::errors::Result<()> {
    let mut client = Client::connect("127.0.0.1:3599").await?;
    let client_ref = &mut client;

    client_ref
        .set("key".into(), ValueType::String("value".into()))
        .await?;

    Ok(())
}
