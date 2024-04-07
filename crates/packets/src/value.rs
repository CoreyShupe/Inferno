use crate::Packet;
use errors::InfernoError;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ValueType {
    None,
    UInt(u32),
    String(String),
}

impl Packet for ValueType {
    async fn write<W>(&self, stream: &mut W) -> errors::Result<()>
    where
        W: AsyncWrite + Unpin,
    {
        match self {
            ValueType::None => {
                stream.write_u8(0).await?;
                Ok(())
            }
            ValueType::UInt(uint) => {
                stream.write_u8(1).await?;
                stream.write_u32(*uint).await?;
                Ok(())
            }
            ValueType::String(string) => {
                stream.write_u8(2).await?;
                string.write(stream).await?;
                Ok(())
            }
        }
    }

    async fn read<R>(stream: &mut R) -> errors::Result<Self>
    where
        R: AsyncRead + Unpin,
    {
        let value = stream.read_u8().await?;
        match value {
            0 => Ok(ValueType::None),
            1 => Ok(ValueType::UInt(stream.read_u32().await?)),
            2 => Ok(ValueType::String(String::read(stream).await?)),
            _ => Err(InfernoError::Packets(
                errors::PacketsError::UnknownValueType(value),
            )),
        }
    }
}
