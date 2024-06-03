use flate2::read::GzDecoder;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use std::io::Cursor;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::{Arc, LazyLock};
use tar::Archive;
use tokio::fs;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{ChildStdin, ChildStdout, Command};
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    let data = include_bytes!("../assets/aws-cdk-lib-2.144.0.tgz");
    let target_dir = "cdk-target";

    match fs::create_dir(target_dir).await {
        Ok(_) => {
            let cursor = Cursor::new(data);
            let gz_decoder = GzDecoder::new(cursor);

            let mut archive = Archive::new(gz_decoder);
            archive.unpack(target_dir).unwrap();
        }
        Err(e) if e.kind() == ErrorKind::AlreadyExists => {}
        Err(e) => todo!("{:?}", e),
    }

    let mut file_path = PathBuf::from(target_dir);
    file_path.push("worker.js");
    let worker_js = include_str!("../worker.js");
    fs::write(&file_path, worker_js).await.unwrap();

    let mut file_path = PathBuf::from(target_dir);
    file_path.push("package.json");
    let package_json = include_str!("../package.json");
    fs::write(&file_path, package_json).await.unwrap();

    let mut file_path = PathBuf::from(target_dir);
    file_path.push("cdk.json");
    let package_json = include_str!("../cdk.json");
    fs::write(&file_path, package_json).await.unwrap();

    Command::new("npm.cmd")
        .arg("install")
        .current_dir(&target_dir)
        .spawn()
        .unwrap()
        .wait()
        .await
        .unwrap();

    let mut p = PathBuf::from("target");
    p.push("debug");
    p.push("examples");
    p.push("app.exe");

    let mut output_path = PathBuf::from(target_dir);
    output_path.push("app.exe");
    
    fs::copy(p, output_path).await.unwrap();
}
