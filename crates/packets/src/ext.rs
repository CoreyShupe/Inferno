use crate::{Packet, PacketDelegate};
use errors::InfernoError;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

impl<K, V> Packet for (K, V)
where
    K: Packet,
    V: Packet,
{
    async fn write<W>(&self, stream: &mut W) -> errors::Result<()>
    where
        W: AsyncWrite + Unpin,
    {
        self.0.write(stream).await?;
        self.1.write(stream).await?;
        Ok(())
    }

    async fn read<R>(stream: &mut R) -> errors::Result<Self>
    where
        R: AsyncRead + Unpin,
    {
        Ok((K::read(stream).await?, V::read(stream).await?))
    }
}

impl Packet for u32 {
    async fn write<W>(&self, stream: &mut W) -> errors::Result<()>
    where
        W: AsyncWrite + Unpin,
    {
        stream.write_u32(*self).await?;
        Ok(())
    }

    async fn read<R>(stream: &mut R) -> errors::Result<Self>
    where
        R: AsyncRead + Unpin,
    {
        let value = stream.read_u32().await?;
        Ok(value)
    }
}

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

impl<T: Packet> Packet for Vec<T> {
    async fn write<W>(&self, stream: &mut W) -> errors::Result<()>
    where
        W: AsyncWrite + Unpin,
    {
        stream.write_u32(self.len() as u32).await?;
        for packet in self {
            packet.write(stream).await?;
        }
        Ok(())
    }

    async fn read<R>(stream: &mut R) -> errors::Result<Self>
    where
        R: AsyncRead + Unpin,
    {
        let length = stream.read_u32().await?;
        let mut packets = Vec::with_capacity(length as usize);
        for _ in 0..length {
            packets.push(<T as Packet>::read(stream).await?);
        }
        Ok(packets)
    }
}

impl<T: Packet> Packet for Option<T> {
    async fn write<W>(&self, stream: &mut W) -> errors::Result<()>
    where
        W: AsyncWrite + Unpin,
    {
        match self {
            Some(value) => {
                stream.write_u8(1).await?;
                value.write(stream).await?;
            }
            None => {
                stream.write_u8(0).await?;
            }
        }
        Ok(())
    }

    async fn read<R>(stream: &mut R) -> errors::Result<Self>
    where
        R: AsyncRead + Unpin,
    {
        let value = stream.read_u8().await?;
        match value {
            0 => Ok(None),
            1 => Ok(Some(<T as Packet>::read(stream).await?)),
            _ => Err(InfernoError::Packets(
                errors::PacketsError::UnknownValueType(value),
            )),
        }
    }
}
