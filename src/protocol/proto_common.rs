use crate::protocol::pool::connection_manager::TcpManager;
use deadpool::managed::Object;
use std::io;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;

pub static TRACKER_PROTO_CMD_SERVICE_QUERY_STORE_WITHOUT_GROUP_ALL:u8 = 106;

pub static TRACKER_PROTO_CMD_SERVICE_QUERY_STORE_WITH_GROUP_ALL:u8 = 107;

pub static FDFS_GROUP_NAME_MAX_LEN:u32 = 16;

pub static FDFS_PROTO_PKG_LEN_SIZE:usize = 8;

pub static PROTO_HEADER_CMD_INDEX:usize = FDFS_PROTO_PKG_LEN_SIZE;

pub static PROTO_HEADER_STATUS_INDEX:usize = FDFS_PROTO_PKG_LEN_SIZE + 1;

pub static TRACKER_PROTO_CMD_RESP:u8 = 100;

pub static FDFS_IPADDR_SIZE:usize = 46;


pub static STORAGE_PROTO_CMD_UPLOAD_FILE:u8 = 11;

pub static FDFS_FILE_EXT_NAME_MAX_LEN:usize = 6;


pub static TRACKER_QUERY_STORAGE_STORE_BODY_LEN:usize = FDFS_GROUP_NAME_MAX_LEN as usize + FDFS_IPADDR_SIZE + FDFS_PROTO_PKG_LEN_SIZE;

pub static STORAGE_PROTO_CMD_RESP:u8 = TRACKER_PROTO_CMD_RESP;

pub static FDFS_PROTO_CMD_ACTIVE_TEST:u8 = 111;




pub fn pack_header(cmd:u8, pkg_len: u64, errno: u8) -> Vec<u8>{
    let mut header = vec![0u8;FDFS_PROTO_PKG_LEN_SIZE + 2];
    let hex_len = long2buff(pkg_len);

    if FDFS_PROTO_PKG_LEN_SIZE < 8 {
        panic!("请检查协议是否更改，跟原有协议不同");
    }
    header[0..hex_len.len()].copy_from_slice(&hex_len);


    header[PROTO_HEADER_CMD_INDEX] = cmd;
    header[PROTO_HEADER_STATUS_INDEX] = errno;
    header
}



/**
 * long convert to buff (big-endian)
 *
 * @param n long number
 * @return 8 bytes buff
 */

pub fn long2buff(n:u64) -> Vec<u8>{
    let mut bs = vec![0u8;8];
    bs[0] = (n >> 56) as u8;
    bs[1] = (n >> 48) as u8;
    bs[2] = (n >> 40) as u8;
    bs[3] = (n >> 32) as u8;
    bs[4] = (n >> 24) as u8;
    bs[5] = (n >> 16) as u8;
    bs[6] = (n >> 8) as u8;
    bs[7] = n as u8;
    bs
}


pub struct RecvPackageInfo{
    pub errno:u8,
    pub body: Vec<u8>,
}

#[derive(Debug)]
pub struct RecvHeaderInfo{
    pub errno:u8,
    pub body_len: usize,
}

pub async fn recv_package(input: &mut Object<TcpManager>,expect_cmd:u8,expect_body_len:Option<usize>) -> Result<RecvPackageInfo,io::Error>{
    let header = recv_header(input, expect_cmd, expect_body_len).await?;
        if header.errno != 0{
        return Err(io::Error::new(io::ErrorKind::Other,format!("recv errno: {} is not correct, expect errno: 0",header.errno)));
    }
    let mut body = vec![0u8;header.body_len];
    input.read(&mut body).await?;
    Ok(RecvPackageInfo { errno:0, body })
}

pub async fn recv_header(input: &mut TcpStream, expect_cmd: u8, expect_body_len: Option<usize>) -> Result<RecvHeaderInfo,io::Error>{
    let mut header = vec![0u8;FDFS_PROTO_PKG_LEN_SIZE+2];
    let recv_len = input.read(&mut header).await?;
    if recv_len != header.len() {
        return Err(io::Error::new(io::ErrorKind::UnexpectedEof,"读取头部信息失败,长度不足!"));
    }

    // 检查命令字节
    if header[PROTO_HEADER_CMD_INDEX] != expect_cmd {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "recv cmd: {} is not correct, expect cmd: {}",
                header[PROTO_HEADER_CMD_INDEX], expect_cmd
            ),
        ));
    }

    // 检查状态字节
    let status = header[PROTO_HEADER_STATUS_INDEX];
    if status != 0 {
        return Ok(RecvHeaderInfo { errno:status, body_len: 0 });
    }

        let recv_len = buff2long(&header[0..8], 0);
        if recv_len < 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("recv body length: {} < 0!", recv_len),
        ));
    }
    let pkg_len = recv_len as usize;
    if let Some(expect_len) = expect_body_len {
        if expect_len != pkg_len {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "recv body length: {} is not correct, expect length: {:?}",
                    pkg_len, expect_len
                ),
            ));
        }
    }

    Ok(RecvHeaderInfo { errno: 0, body_len: pkg_len })

}

pub fn buff2long(bs: &[u8], offset: usize) -> u64 {
    ((bs[offset] as u64) << 56) |
        ((bs[offset + 1] as u64) << 48) |
        ((bs[offset + 2] as u64) << 40) |
        ((bs[offset + 3] as u64) << 32) |
        ((bs[offset + 4] as u64) << 24) |
        ((bs[offset + 5] as u64) << 16) |
        ((bs[offset + 6] as u64) << 8) |
        (bs[offset + 7] as u64)
}
