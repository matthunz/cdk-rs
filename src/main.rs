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

#[derive(Serialize)]
struct Request<'a> {
    js: &'a str,
}

#[derive(Deserialize)]
struct Response {
    json: Value,
}

static WORKER: LazyLock<Arc<Mutex<Option<Worker>>>> = LazyLock::new(|| Arc::new(Mutex::new(None)));

struct Worker {
    stdin: ChildStdin,
    stdout: ChildStdout,
}

pub async fn request<T>(js: &str) -> T
where
    T: for<'de> Deserialize<'de>,
{
    let mut worker_cell = WORKER.lock().await;
    let worker = if let Some(worker) = &mut *worker_cell {
        worker
    } else {
        let data = include_bytes!("../assets/aws-cdk-lib-2.144.0.tgz");
        let target_dir = "cdk-target";

        match fs::create_dir("path").await {
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
        file_path.push("cdk-rs.js");

        let worker_js = include_str!("../worker.js");
        fs::write(&file_path, worker_js).await.unwrap();

        let mut child = Command::new("node")
            .arg("cdk-rs.js")
            .current_dir(&target_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed to start child process");

        let stdin = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();

        tokio::task::spawn(async move {
            child.wait().await.unwrap();
        });

        *worker_cell = Some(Worker { stdin, stdout });
        worker_cell.as_mut().unwrap()
    };

    let message = Request { js };
    let mut bytes = serde_json::to_vec(&message).unwrap();
    bytes.push(b'\n');
    worker.stdin.write_all(&bytes).await.unwrap();

    let reader = BufReader::new(&mut worker.stdout);
    let mut lines = reader.lines();

    let Ok(Some(line)) = lines.next_line().await else {
        todo!()
    };

    let res: Response = serde_json::from_str(&line).unwrap();
    serde_json::from_value(res.json).unwrap()
}

#[tokio::main]
async fn main() {
    dbg!(request::<i32>("2 + 2").await);
}
