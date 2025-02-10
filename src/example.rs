use std::io;
use std::path::Path;
use dlgump_fastdfs_client_rs::protocol::storage_client::StorageClient;

#[tokio::test]
async fn example_upload() -> Result<(), io::Error>{
    let location = "C:\\Users\\dlgump\\Desktop\\国际摩尔斯电码.svg.png";
    let path = Path::new(location);
    let file_ext_name = path.extension().unwrap().to_str().unwrap().to_string();
    let file = tokio::fs::read(path).await?;
    println!("{:?}",StorageClient::upload_file(file, &file_ext_name).await?);
    let file = tokio::fs::read(path).await?;
    println!("{:?}",StorageClient::upload_file(file, &file_ext_name).await?);
    Ok(())
}