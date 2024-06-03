use flate2::read::GzDecoder;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use tokio::task::JoinHandle;
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

pub struct App {
    stdin: ChildStdin,
    stdout: ChildStdout,
    handle: JoinHandle<()>
}

impl App {
    pub async fn new() -> Self {
        let mut child = Command::new("node")
            .arg("worker.js")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed to start child process");

        let stdin = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();

        let handle = tokio::task::spawn(async move {
            child.wait().await.unwrap();
        });

        let mut me = App { stdin, stdout, handle };
        me.request::<i32>(
            r#"
                app = new cdk.App();
                
                class HelloCdkStack extends cdk.Stack {
                    constructor(scope, id, props) {
                      super(scope, id, props);
                  
                      new s3.Bucket(this, 'MyFirstBucket', {
                        versioned: true
                      });
                    }
                }

                new HelloCdkStack(app, 'HelloCdkStack', {});
                
                0
            "#,
        )
        .await;
        me
    }

    pub async fn request<T>(&mut self, js: &str) -> T
    where
        T: for<'de> Deserialize<'de>,
    {
        let message = Request { js };
        let mut bytes = serde_json::to_vec(&message).unwrap();
        bytes.push(b'\n');
        self.stdin.write_all(&bytes).await.unwrap();

        let reader = BufReader::new(&mut self.stdout);
        let mut lines = reader.lines();

        let Ok(Some(line)) = lines.next_line().await else {
            todo!()
        };

        let res: Response = serde_json::from_str(&line).unwrap();
        serde_json::from_value(res.json).unwrap()
    }
}

impl Drop for App {
    fn drop(&mut self) {
        self.handle.abort();
    }
}