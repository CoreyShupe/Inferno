use crate::{Packet, PacketDelegate};
use errors::InfernoError;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

impl Packet for String {
    async fn write<W>(&self, stream: &mut W) -> errors::Result<()>
    where
        W: AsyncWrite + Unpin,
    {
        stream.write_u32(self.len() as u32).await?;
        stream.write_all(self.as_bytes()).await?;
        Ok(())
    }

    async fn read<R>(stream: &mut R) -> errors::Result<Self>
    where
        R: AsyncRead + Unpin,
    {
        let length = stream.read_u32().await?;
        let mut buf = vec![0u8; length as usize];
        stream.read_exact(&mut buf).await?;
        Ok(String::from_utf8(buf)?)
    }
}

impl PacketDelegate for InfernoError {
    type PacketType = String;

    fn packet(&self) -> Self::PacketType {
        self.to_string()
    }

    fn delegate(packet: Self::PacketType) -> Self {
        Self::DecodedMessage(packet)
    }
}
