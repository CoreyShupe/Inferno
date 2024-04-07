pub mod ext;
pub(crate) mod macros;
pub mod value;

use crate::value::ValueType;
use errors::InfernoError;
use errors::Result;
use std::future::Future;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

pub trait Packet: Sized {
    fn write<W>(&self, stream: &mut W) -> impl Future<Output = Result<()>>
    where
        W: AsyncWrite + Unpin;

    fn read<R>(stream: &mut R) -> impl Future<Output = Result<Self>>
    where
        R: AsyncRead + Unpin;
}

pub trait PacketDelegate: Sized {
    type PacketType: Packet;

    fn packet(&self) -> Self::PacketType;

    fn delegate(packet: Self::PacketType) -> Self;
}

impl<T: PacketDelegate> Packet for T {
    async fn write<W>(&self, stream: &mut W) -> Result<()>
    where
        W: AsyncWrite + Unpin,
    {
        self.packet().write(stream).await
    }

    async fn read<R>(stream: &mut R) -> Result<Self>
    where
        R: AsyncRead + Unpin,
    {
        let packet = <Self as PacketDelegate>::PacketType::read(stream).await?;
        Ok(T::delegate(packet))
    }
}

macro_rules! packet_types {
    ($packet_enum:ident { $($name:ident$({
        $($field:ident: $field_type:ty),*$(,)?
    })? = $value:literal),*$(,)? }) => {
        #[derive(Debug)]
        pub enum $packet_enum {
            $(
                $name$({
                    $($field: $field_type),*
                })?,
            )*
        }

        impl Packet for $packet_enum {
            async fn write<W>(&self, stream: &mut W) -> Result<()>
            where
                W: AsyncWrite + Unpin,
            {
                match self {
                    $(
                        #[allow(non_snake_case)]
                        $packet_enum::$name$({
                            $($field),*
                        })? => {
                            stream.write_u8($value).await?;
                            $(
                            $(
                                $field.write(stream).await?;
                            )*
                            )?
                            Ok(())
                        },
                    )*
                }
            }

            async fn read<R>(stream: &mut R) -> Result<Self>
            where
                R: AsyncRead + Unpin,
            {
                let value = stream.read_u8().await?;
                match value {
                    $(
                        $value => {
                            $(
                                $(let $field = <$field_type as $crate::Packet>::read(stream).await?;)*
                            )?
                            Ok(
                                Self::$name $({
                                    $($field),*
                                })?
                            )
                        },
                    )*
                    _ => Err(InfernoError::Packets(errors::PacketsError::UnknownPacketType(value))),
                }
            }
        }
    }
}

packet_types! {
    ClientCommand {
        Set { key: String, value: ValueType } = 0x00,
        Get { key: String } = 0x01,
        Del { key: String } = 0x02,
    }
}

packet_types! {
    ServerResponse {
        Error { err: InfernoError } = 0x00,
        Ok = 0x01,
        OkSingle { value: ValueType } = 0x02,
    }
}
