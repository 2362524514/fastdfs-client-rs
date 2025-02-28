use std::io;
use std::path::Path;
use fastdfs_client_rs::config::settings::get_settings;
use fastdfs_client_rs::protocol::storage_client::StorageClient;

#[tokio::main]
async fn main() -> Result<(), io::Error>{
    //如果需要定制化配置文件，得提前通过get_settings进行初始化
    get_settings(Some("fastdfs.conf"));
    // get_settings(Some("/data/jar/police_data_deal_bin/fastdfs.conf"));
    let location = "C:\\Users\\dlgump\\Desktop\\国际摩尔斯电码.svg.png";
    let path = Path::new(location);
    let file_ext_name = path.extension().unwrap().to_str().unwrap().to_string();
    let file = tokio::fs::read(path).await?;

    println!("{:?}",StorageClient::upload_file(&file, "jpg").await?);
    // let file = tokio::fs::read(path).await?;
    // println!("{:?}",StorageClient::upload_file(&file, &file_ext_name).await?);
    Ok(())
}