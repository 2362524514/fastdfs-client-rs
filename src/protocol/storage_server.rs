
#[derive(Debug)]
pub struct StorageServer {
    pub ip: String,
    pub port: u16,
    pub storage_path: u8,
}


impl StorageServer {
    pub fn new(ip:&str,port:u16,storage_path: u8) -> Self {
        StorageServer{
            ip: ip.to_string(),port,storage_path
        }
    }
}