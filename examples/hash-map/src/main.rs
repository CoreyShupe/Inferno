use inferno::driver::{Client, CommandSender, Instruction, ServerResponse, ValueType};

#[tokio::main]
async fn main() -> inferno::errors::Result<()> {
    let mut client = Client::connect("127.0.0.1:3599").await?;

    client
        .del("key", vec![Instruction::hash("hash-key")])
        .await?;

    client
        .set(
            "key",
            vec![Instruction::hash("hash-key"), Instruction::hash("nested")],
            ValueType::UInt(1),
        )
        .await?;

    let value = client
        .get(
            "key",
            vec![Instruction::hash("hash-key"), Instruction::hash("nested")],
        )
        .await?;

    println!("Value: {:#?}", value);

    assert!(matches!(
        value,
        ServerResponse::OkSingle {
            value: ValueType::UInt(1)
        }
    ));

    Ok(())
}
