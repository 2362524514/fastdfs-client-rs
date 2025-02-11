use crate::config::settings::get_settings;
use crate::protocol::pool::connection_manager::TcpManager;
use crate::protocol::pool::connection_pool::{get_connection_pool};
use crate::protocol::proto_common;
use crate::protocol::proto_common::pack_header;
use crate::protocol::storage_server::StorageServer;
use deadpool::managed::Object;
use std::io;
use std::io::ErrorKind;
use tokio::io::AsyncWriteExt;

pub struct TrackerServer;

impl TrackerServer {
    pub async fn get_tracker_server_connection() -> Result<Object<TcpManager>,io::Error>{
        // let tracker_servers = &SETTINGS.read().await.tracker_server;
        let tracker_servers = &get_settings(None).tracker_server;
        let len = tracker_servers.len();
        if len == 0 {
            return Err(io::Error::new(io::ErrorKind::Other,"tracker_server配置不存在!"));
        }
        let random_start_index = (rand::random::<u16>() as usize) % len;
        for i in 0..len {
            let index = (random_start_index + i) % len ;
            let tracker_server = &tracker_servers[index];
            let result = get_connection_pool().get_connection(tracker_server).await;
            if let Ok(connection) = result {
                return Ok(connection);
            }
        }
        Err(io::Error::new(io::ErrorKind::Other,"无可用tracker_server！"))
    }



    pub async fn get_storage_servers(group_name:Option<&str>) -> Result<Vec<StorageServer>, io::Error> {
        let tracker_server_res = Self::get_tracker_server_connection().await;
        if tracker_server_res.is_err(){
            return Err(tracker_server_res.unwrap_err());
        }
        let mut tracker_stream = tracker_server_res.unwrap();
        let cmd;
        let out_len;
        if group_name.is_none() || group_name.unwrap().len() == 0usize {
            cmd = proto_common::TRACKER_PROTO_CMD_SERVICE_QUERY_STORE_WITHOUT_GROUP_ALL;
            out_len = 0u64;
        } else {
            cmd = proto_common::TRACKER_PROTO_CMD_SERVICE_QUERY_STORE_WITH_GROUP_ALL;
            out_len = proto_common::FDFS_GROUP_NAME_MAX_LEN as u64;
        }
        let header = pack_header(cmd, out_len, 0u8);
        if let Err(msg) = tracker_stream.write(&header).await {
            return Err(msg);
        }

        if let Some(group_name_str) = group_name  {
            if group_name_str.len() != 0{
                //将group_name_str根据UTF-8转化为byte数组
                let bs = group_name_str.as_bytes();
                let mut group_name_bytes = vec![0u8;proto_common::FDFS_GROUP_NAME_MAX_LEN as usize];

                let group_len = if bs.len() <= proto_common::FDFS_GROUP_NAME_MAX_LEN as usize {
                    bs.len()
                } else {
                    proto_common::FDFS_GROUP_NAME_MAX_LEN as usize
                };
                //System.arraycopy(bs, 0, bGroupName, 0, group_len);
                group_name_bytes[..group_len].copy_from_slice(&bs[..group_len]);
                if let Err(msg) = tracker_stream.write(&group_name_bytes).await {
                    return Err(msg);
                }
            }
        }


        let pkg_info  = proto_common::recv_package(&mut tracker_stream,proto_common::TRACKER_PROTO_CMD_RESP,None).await?;
        if pkg_info.errno != 0 {
            return Err(io::Error::new(ErrorKind::Other,format!("tracker_server返回错误:{}",pkg_info.errno)));
        }

        if pkg_info.body.len() < proto_common::TRACKER_QUERY_STORAGE_STORE_BODY_LEN {
            return Err(io::Error::new(ErrorKind::AddrNotAvailable,"无效参数!"));
        }
        let ip_port_len = pkg_info.body.len() - (proto_common::FDFS_GROUP_NAME_MAX_LEN as usize + 1);
        let record_length = proto_common::FDFS_IPADDR_SIZE - 1 + proto_common::FDFS_PROTO_PKG_LEN_SIZE;

        if ip_port_len % record_length != 0 {
            return Err(io::Error::new(ErrorKind::AddrNotAvailable,"无效参数2!"));
        }

        let server_count = ip_port_len / record_length;
        if server_count > 16{
            return Err(io::Error::new(ErrorKind::AddrNotAvailable,"磁盘无空闲空间!"));
        }
        let mut storage_servers = Vec::with_capacity(server_count);
        let storage_path = pkg_info.body[pkg_info.body.len() - 1];
        let mut offset = proto_common::FDFS_GROUP_NAME_MAX_LEN as usize;
        for i in 0..server_count {
            let x = &pkg_info.body[offset..(offset + proto_common::FDFS_IPADDR_SIZE - 1)];
            let ip = String::from(String::from_utf8_lossy(x).trim_end_matches('\0'));
            offset += (proto_common::FDFS_IPADDR_SIZE - 1);
            let port = proto_common::buff2long(&pkg_info.body,offset) as u16;
            offset += proto_common::FDFS_PROTO_PKG_LEN_SIZE;
            storage_servers.push(StorageServer{ip,port,storage_path});
        }
        Ok(storage_servers)
    }



}