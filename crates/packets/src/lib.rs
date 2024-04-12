#![feature(macro_metavar_expr)]

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
    (@executor $me:ident -> $executor:ident { $($name:ident as $fn_ident:ident $({
        $($field:ident: $field_type:ty),*$(,)?
    })?),*$(,)? } -> $return_type:ty) => {
        pub trait $executor: Sized {
            $(
                fn $fn_ident(self$(, $($field: $field_type),*)?) -> impl Future<Output = Result<$return_type>>;
            )*
        }

        impl<T> $executor for T where T: PacketSender<$me, $return_type> {
            $(
                async fn $fn_ident(self$(, $($field: $field_type),*)?) -> Result<$return_type> {
                    self.send(&$me::$name$({
                        $($field),*
                    })?).await
                }
            )*
        }
    };
    (@executor $me:ident { $($name:ident $({
        $($field:ident: $field_type:ty),*$(,)?
    })?),*$(,)? }) => {};
    ($packet_enum:ident $(-> $executor:ident)? { $($name:ident $(as $fn_ident:ident)? $({
        $($field:ident: $field_type:ty),*$(,)?
    })?),*$(,)? } $(-> $return_type:ty)?) => {
        #[derive(Debug)]
        pub enum $packet_enum {
            $(
                $name$({
                    $($field: $field_type),*
                })?,
            )*
        }

        packet_types!(@executor $packet_enum $(-> $executor)? { $($name $(as $fn_ident)? $({
            $($field: $field_type),*
        })?),* } $(-> $return_type)?);

        impl Packet for $packet_enum {
            async fn write<W>(&self, stream: &mut W) -> Result<()>
            where
                W: AsyncWrite + Unpin,
            {
                match self {
                    $(
                        $packet_enum::$name$({
                            $($field),*
                        })? => {
                            stream.write_u8(${index()}).await?;
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
                        ${index()} => {
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
    ServerResponse {
        Error { err: InfernoError },
        Ok,
        Single { value: ValueType },
        Bulk { values: Vec<ValueType> },
        OptInt { value: Option<u32> },
        IntList { values: Vec<u32> },
    }
}

packet_types! {
    ClientCommand -> ClientCommandExecutor {
        //// Arbitrary Commands ////

        Expire as expire { key: String, expire: u32 },
        Persist as persist { key: String },
        Ttl as ttl { key: String },
        Del as del { keys: Vec<String> },

        //// Value Commands ////

        Decr as decr { key: String },
        DecrBy as decr_by { key: String, by: u32 },
        Incr as incr { key: String },
        IncrBy as incr_by { key: String, by: u32 },

        Get as get { key: String },
        GetDel as get_del { keys: Vec<String> },
        GetEx as get_ex { key: String, expire: u32 },
        GetSet as get_set { key: String, value: ValueType },
        MGet as mget { keys: Vec<String> },

        Set as set { key: String, value: ValueType },
        SetEx as set_ex { key: String, value: ValueType, expire: u32 },
        SetNx as set_nx { key: String, value: ValueType },
        MSet as mset { keys: Vec<String>, values: Vec<ValueType> },
        MSetNx as mset_nx { keys: Vec<String>, values: Vec<ValueType> },

        //// Hash Commands ////

        HExpire as hexpire { key: String, field: String, expire: u32 },
        HDel as hdel { key: String, fields: Vec<String> },
        HDelGet as hdel_get { key: String, fields: Vec<String> },
        HPopRand as hpop_rand { key: String, count: u32 },

        HExists as hexists { key: String, field: String },
        HGet as hget { key: String, field: String },
        HGetAll as hget_all { key: String },
        HMGet as hmget { key: String, fields: Vec<String> },
        HKeys as hkeys { key: String },
        HValues as hvalues { key: String },
        HLen as hlen { key: String },

        HDecr as hdecr { key: String, field: String },
        HDecrBy as hdecr_by { key: String, field: String },
        HIncr as hincr { key: String, field: String },
        HIncrBy as hincr_by { key: String, field: String, by: u32 },

        HSet as hset { key: String, field: String, value: ValueType },
        HSetNx as hset_nx { key: String, field: String, value: ValueType },
        HSetEx as hset_ex { key: String, field: String, value: ValueType, expire: u32 },
        HMSet as hmset { key: String, fields: Vec<(String, ValueType)> },
        HMSetNx as hmset_nx { key: String, fields: Vec<(String, ValueType)> },

        //// Set Commands ////

        ZAdd as zadd { key: String, score: u32, member: String },
        ZAddNx as zadd_nx { key: String, score: u32, member: String },
        ZIncrBy as zincr_by { key: String, score: u32, member: String },
        ZDecrBy as zdecr_by { key: String, score: u32, member: String },

        ZScore as zscore { key: String, member: String },
        ZMScore as zmscore { key: String, members: Vec<String> },

        ZPopMin as zpop_min { key: String, count: u32 },
        ZPopMax as zpop_max { key: String, count: u32 },

        ZRem as zrem { key: String, member: String },
        ZExpire as zexpire { key: String, member: String, expire: u32 },

        //// List Commands ////

        LLPush as llpush { key: String, value: ValueType },
        LLPushNx as llpush_nx { key: String, value: ValueType },
        LLPushEx as llpush_ex { key: String, value: ValueType, expire: u32 },
        LRPush as lrpush { key: String, value: ValueType },
        LRPushNx as lrpush_nx { key: String, value: ValueType },
        LRPushEx as lrpush_ex { key: String, value: ValueType, expire: u32 },

        LExpire as lexpire { key: String, index: u32, expire: u32 },
        LLPop as llpop { key: String, count: u32 },
        LRPop as lrpop { key: String, count: u32 },
        LRange as lrange { key: String, start: u32, end: u32 },

        //// Set Commands ////

        SAdd as sadd { key: String, members: Vec<String> },
        SAddNx as sadd_nx { key: String, members: Vec<String> },
        SAddEx as sadd_ex { key: String, member: String, expire: u32 },
        SMember as smember { key: String, member: String },
        SMembers as smembers { key: String },

        SExpire as sexpire { key: String, member: String, expire: u32 },
        SRem as srem { key: String, members: Vec<String> },
        SPop as spop { key: String, count: u32 },
    } -> ServerResponse
}

pub trait PacketSender<T, R>: Sized {
    fn send(self, packet: &T) -> impl Future<Output = Result<R>>;
}
