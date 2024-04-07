use inferno::driver::{Client, CommandSender, ServerResponse, ValueType};

#[tokio::main]
async fn main() -> inferno::errors::Result<()> {
    let mut client = Client::connect("127.0.0.1:3599").await?;

    client
        .set("key", ValueType::String("value".to_string()))
        .await?;

    let response = client.get("key").await?;

    println!("{:?}", response);

    if let ServerResponse::OkSingle { value } = response {
        assert_eq!(value, ValueType::String("value".to_string()));
    } else {
        panic!("Expected ServerResponse::OkSingle");
    }

    let response = client.del("key").await?;

    println!("{:?}", response);

    if let ServerResponse::OkSingle { value } = response {
        assert_eq!(value, ValueType::String("value".to_string()));
    } else {
        panic!("Expected ServerResponse::OkSingle");
    }

    Ok(())
}
