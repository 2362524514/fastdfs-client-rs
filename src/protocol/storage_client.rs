use std::io;
use std::io::ErrorKind;
use tokio::io::AsyncWriteExt;
use crate::protocol::pool::connection_pool::{get_connection_pool};
use crate::protocol::proto_common;
use crate::protocol::tracker_server::TrackerServer;

pub struct StorageClient;


impl StorageClient {
    pub async fn upload_file(file_buff: &Vec<u8>, file_ext_name: &str) -> Result<(String,String), io::Error> {
        let file_size = file_buff.len();
        if let Ok(storage_servers) = TrackerServer::get_storage_servers(None).await {
            if storage_servers.len() == 0{
                return Err(io::Error::new(ErrorKind::NotFound,"无可用storage_server"));
            }
            let random_start_index = (rand::random::<u16>() as usize) % storage_servers.len();
            for i in 0..storage_servers.len(){
                let index = (i + random_start_index) % storage_servers.len();
                let storage_server = &storage_servers[index];
                let host = storage_server.ip.to_string() +":"+ &storage_server.port.to_string();
                let mut offset = 0;
                if let Ok(mut stream) = get_connection_pool().get_connection(&host).await{
                    let mut ext_name_bs = vec![0u8;proto_common::FDFS_FILE_EXT_NAME_MAX_LEN];
                    let origin_ext_name_bs = file_ext_name.as_bytes();

                    let ext_name_len = if origin_ext_name_bs.len() > proto_common::FDFS_FILE_EXT_NAME_MAX_LEN {
                        proto_common::FDFS_FILE_EXT_NAME_MAX_LEN
                    }else {
                        origin_ext_name_bs.len()
                    };
                    ext_name_bs[..ext_name_len].copy_from_slice(&origin_ext_name_bs[..ext_name_len]);
                    let mut size_bytes = vec![0u8;proto_common::FDFS_PROTO_PKG_LEN_SIZE + 1];
                    let body_len = size_bytes.len() + proto_common::FDFS_FILE_EXT_NAME_MAX_LEN + file_size;
                    size_bytes[0] = storage_server.storage_path;
                    offset = 1;
                    let hex_len_bytes = proto_common::long2buff(file_size as u64);
                    size_bytes[1..1+hex_len_bytes.len()].copy_from_slice(&hex_len_bytes[..hex_len_bytes.len()]);
                    let header = proto_common::pack_header(proto_common::STORAGE_PROTO_CMD_UPLOAD_FILE, body_len as u64,0u8);
                    let mut whole_pkg = vec![0u8; header.len()+body_len-file_size];
                    whole_pkg[..header.len()].copy_from_slice(&header[..header.len()]);
                    whole_pkg[header.len()..header.len()+ size_bytes.len()].copy_from_slice(&size_bytes[..size_bytes.len()]);
                    offset = header.len() + size_bytes.len();
                    whole_pkg[offset..offset+ext_name_bs.len()].copy_from_slice(&ext_name_bs[..ext_name_bs.len()]);
                    stream.write_all(&whole_pkg).await?;
                    stream.flush().await?;
                    //&whole_pkg转为base64
                    stream.write_all(file_buff).await?;
                    stream.flush().await?;
                    let recv_info = proto_common::recv_package(&mut stream, proto_common::STORAGE_PROTO_CMD_RESP, None).await?;
                    if recv_info.errno != 0 {
                        return Result::Err(io::Error::new(ErrorKind::Other,format!("storage_server返回错误:{}",recv_info.errno)));
                    }
                    if recv_info.body.len() <= proto_common::FDFS_GROUP_NAME_MAX_LEN as usize {
                        return Err(io::Error::new(ErrorKind::Other,"storage_server返回错误:group_name长度不正确!"));
                    }

                    let new_group_name = String::from_utf8_lossy(&recv_info.body[..proto_common::FDFS_GROUP_NAME_MAX_LEN as usize])
                        .trim_end_matches('\0').to_string();
                    let remote_filename = String::from_utf8_lossy(&recv_info.body[proto_common::FDFS_GROUP_NAME_MAX_LEN as usize..]).trim().to_string();
                    return Ok((new_group_name,remote_filename));
                }


            }


            Err(io::Error::new(ErrorKind::Other,"所有storage_server获取连接均失败!"))
        }else{
            Err(io::Error::new(ErrorKind::Other,"获取storage_server失败!"))
        }
    }
}