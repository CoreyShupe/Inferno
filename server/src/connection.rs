use crate::state::State;
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

        macro_rules! response_handler {
            ($command:ident($state:ident) -> { $($big:ident as $small:ident { $($field:ident),*$(,)? })* } -> $return_ident:ident) => {
                use packets::ClientCommandExecutor;
                let $return_ident = match $command {
                    $(
                        ClientCommand::$big { $($field),* } => $state.$small($($field),*).await,
                    )*
                };
            }
        }

        response_handler!(
            command(state) -> {
                Expire as expire { key, expire }
                Persist as persist { key }
                Ttl as ttl { key }
                Del as del { keys }

                //// Value Commands ////

                Decr as decr { key }
                DecrBy as decr_by { key, by }
                Incr as incr { key }
                IncrBy as incr_by { key, by }

                Get as get { key }
                GetDel as get_del { keys }
                GetEx as get_ex { key, expire }
                GetSet as get_set { key, value }
                MGet as mget { keys }

                Set as set { key, value }
                SetEx as set_ex { key, value, expire }
                SetNx as set_nx { key, value }
                MSet as mset { keys, values }
                MSetNx as mset_nx { keys, values }

                //// Hash Commands ////

                HExpire as hexpire { key, field, expire }
                HDel as hdel { key, fields }
                HDelGet as hdel_get { key, fields }
                HPopRand as hpop_rand { key, count }

                HExists as hexists { key, field }
                HGet as hget { key, field }
                HGetAll as hget_all { key }
                HMGet as hmget { key, fields }
                HKeys as hkeys { key }
                HValues as hvalues { key }
                HLen as hlen { key }

                HDecr as hdecr { key, field }
                HDecrBy as hdecr_by { key, field }
                HIncr as hincr { key, field }
                HIncrBy as hincr_by { key, field, by }

                HSet as hset { key, field, value }
                HSetNx as hset_nx { key, field, value }
                HSetEx as hset_ex { key, field, value, expire }
                HMSet as hmset { key, fields }
                HMSetNx as hmset_nx { key, fields }

                //// Set Commands ////

                ZAdd as zadd { key, score, member }
                ZAddNx as zadd_nx { key, score, member }
                ZIncrBy as zincr_by { key, score, member }
                ZDecrBy as zdecr_by { key, score, member }

                ZScore as zscore { key, member }
                ZMScore as zmscore { key, members }

                ZPopMin as zpop_min { key, count }
                ZPopMax as zpop_max { key, count }

                ZRem as zrem { key, member }
                ZExpire as zexpire { key, member, expire }

                //// List Commands ////

                LLPush as llpush { key, value }
                LLPushNx as llpush_nx { key, value }
                LLPushEx as llpush_ex { key, value, expire }
                LRPush as lrpush { key, value }
                LRPushNx as lrpush_nx { key, value }
                LRPushEx as lrpush_ex { key, value, expire }

                LExpire as lexpire { key, index, expire }
                LLPop as llpop { key, count }
                LRPop as lrpop { key, count }
                LRange as lrange { key, start, end }

                //// Set Commands ////

                SAdd as sadd { key, members }
                SAddNx as sadd_nx { key, members }
                SAddEx as sadd_ex { key, member, expire }
                SMember as smember { key, member }
                SMembers as smembers { key }

                SExpire as sexpire { key, member, expire }
                SRem as srem { key, members }
                SPop as spop { key, count }
            } -> response
        );

        match response {
            Ok(response) => response.write(&mut write).await?,
            Err(err) => ServerResponse::Error { err }.write(&mut write).await?,
        }
    }
}
