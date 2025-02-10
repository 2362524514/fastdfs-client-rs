use deadpool::managed::{Manager, Metrics, RecycleResult};
use std::io::Error;
use std::time::Duration;
use tokio::io;
use tokio::net::TcpStream;
use tokio::time::timeout;

/// 自定义 TCP 连接管理器，用于为指定目标地址建立 TCP 连接，并设置连接超时时间
#[derive(Debug)]
pub struct TcpManager {
    target: String,
    connection_timeout: Duration,
}

impl TcpManager {
    pub fn new(target: String, connection_timeout: Duration) -> Self {
        Self {
            target,
            connection_timeout,
        }
    }
}

// #[async_trait]
impl Manager for TcpManager {
    type Type = TcpStream;
    type Error = Error;

    async fn create(&self) -> Result<TcpStream, Error> {
        /// 创建 TCP 连接时，通过 timeout 限制建立连接的时间
        let stream = timeout(self.connection_timeout, TcpStream::connect(&self.target)).await??;
        Ok(stream)
    }

    /// recycle 方法可以用于检测连接是否健康（这里示例简单返回 Ok(()); 实际可添加读写检测）
    async fn recycle(&self, conn: &mut TcpStream, _metrics: &Metrics) -> RecycleResult<io::Error> {
        println!("重新连接");
        Ok(())
    }
}