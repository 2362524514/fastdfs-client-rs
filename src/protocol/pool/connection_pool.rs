use crate::protocol::pool::connection_manager::TcpManager;
use dashmap::DashMap;
use deadpool::managed::{Manager, Object, Pool, PoolConfig, PoolError, Timeouts};
use std::io::Error;
use deadpool_runtime::Runtime;
use tokio::time::Duration;

lazy_static!{


     pub static ref CONNECTION_POOL:MultiTargetPool = {
        let  settings = tokio::task::block_in_place(|| SETTINGS.blocking_read().clone());
        let connect_timeout = settings.connect_timeout;
        let idel_timeout = settings.connection_pool.max_idle_time;
        let max_lifetime = settings.connection_pool.max_wait_time_in_ms;
        MultiTargetPool::new(
            16,
            Duration::from_secs(connect_timeout),
            Some(Duration::from_secs(idel_timeout)),
            Some(Duration::from_secs(max_lifetime)),
        )
    };
}


/// MultiTargetPool 用于管理多个目标地址对应的 deadpool 池，内部使用 DashMap 实现映射
pub struct MultiTargetPool {
    /// key 为目标地址，value 为针对该地址的 deadpool 池
    pub pools: DashMap<String, Pool<TcpManager>>,
    /// 连接池的全局配置
    idle_timeout: Option<Duration>,
    max_lifetime: Option<Duration>,
    connection_timeout: Duration,
    max_size: usize,
}

impl MultiTargetPool {
    /// 构造一个 MultiTargetPool，指定每个池的最大连接数、连接建立超时、闲置超时和最大生命周期
    pub fn new(
        max_size: usize,
        connection_timeout: Duration,
        idle_timeout: Option<Duration>,
        max_lifetime: Option<Duration>,
    ) -> Self {
        Self {
            pools: DashMap::new(),
            idle_timeout,
            max_lifetime,
            connection_timeout,
            max_size,
        }
    }

    /// 获取指定目标地址的连接
    pub async fn get_connection(&self, target: &str) -> Result<Object<TcpManager>, PoolError<Error>> {
        // 如果池已存在，直接使用
        if let Some(pool) = self.pools.get(target) {
            return pool.get().await;
        }
        // 如果池不存在，则新建一个管理器与连接池
        let manager = TcpManager::new(target.to_string(), self.connection_timeout);
        // 构造 deadpool 的池配置（idle_timeout 和 max_lifetime 单位通常为秒，这里转换为 u64 秒数）
        let pool_config = PoolConfig {
            max_size: self.max_size,
            timeouts: Timeouts {
                wait: self.idle_timeout,
                create: Some(self.connection_timeout),
                recycle: Some(Duration::from_secs(10)),
                ..Default::default()
            },
            ..Default::default()
        };
        // 使用 Tokio 运行时构造池（这里使用 deadpool::Runtime::Tokio1）
        let pool = Pool::builder(manager)
            .config(pool_config)
            .runtime(Runtime::Tokio1)
            .build()
            .unwrap();
        self.pools.insert(target.to_string(), pool);
        // 再次获取对应目标的连接
        self.pools.get(target).unwrap().get().await
    }
}



















use std::io;
use std::sync::Arc;
use lazy_static::lazy_static;
use crate::config::settings::SETTINGS;

#[tokio::test]
async fn test() -> Result<(), io::Error> {
    // 构造 MultiTargetPool，设置每个池最大连接数为 8，
    // 连接建立超时为 5 秒，闲置超时为 60 秒，最大生命周期为 300 秒
    let multi_pool = Arc::new(MultiTargetPool::new(
        8,
        Duration::from_secs(5),
        Some(Duration::from_secs(60)),
        Some(Duration::from_secs(300)),
    ));

    // 示例目标地址列表
    let targets = vec!["127.0.0.1:80"
                       // , "127.0.0.1:8001"
                       // , "127.0.0.1:8002"
    ];

    let mut handles = vec![];
    for i in 0..10{
        for target in targets.clone() {
            let pool = multi_pool.clone();
            let target = target.to_string();
            let handle = tokio::spawn(async move {
                match pool.get_connection(&target).await {
                    Ok(conn) => {
                        println!("获得目标{} {} 的连接",i, target);
                        // 使用连接进行一些操作，这里仅示例获取后即归还（deadpool 自动归还连接）
                        drop(conn);
                        // sleep(Duration::from_secs(2))
                    }
                    Err(e) => {
                        eprintln!("连接 {} 失败：{}", target, e);
                    }
                }
            });
            handles.push(handle);
        }
    }


    for handle in handles {
        handle.await.unwrap();
    }

    // 输出各目标连接池当前的连接数量
    for entry in multi_pool.pools.iter() {
        println!(
            "目标 {} 的连接池中有 {} 个连接，最大连接数为{}",
            entry.key(),
            entry.value().status().size,
            entry.value().status().max_size
        );
    }

    Ok(())
}


