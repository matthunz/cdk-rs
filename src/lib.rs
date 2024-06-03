use flate2::read::GzDecoder;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use std::borrow::Cow;
use std::cell::RefCell;
use std::future::Future;
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
use tokio::task::JoinHandle;

pub mod s3;

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
    handle: JoinHandle<()>,
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

        let mut me = App {
            stdin,
            stdout,
            handle,
        };
        me.request::<i32>(
            r#"
                app = new cdk.App();
                0
            "#,
        )
        .await;
        me
    }

    pub async fn stack(&mut self, mut stack: impl Stack) {
        StackContext::set(StackContext::default());

        stack.stack(self);

        let cx = StackContext::get();
        let exprs = cx.exprs.concat();
        let js = &format!(
            r#"
                class RustStack extends cdk.Stack {{
                    constructor(scope, id, props) {{
                        super(scope, id, props);
                        {}
                    }}
                }}

                new RustStack(app, '{}', {{}});

                0
            "#,
            exprs,
            stack.name()
        );
        self.request::<Value>(js).await;

    }

    async fn request<T>(&mut self, js: &str) -> T
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

thread_local! {
    static STACK_CONTEXT: RefCell<Option<StackContext>> = RefCell::new(None);
}

#[derive(Default)]
struct StackContext {
    exprs: Vec<Cow<'static, str>>,
}

impl StackContext {
    fn set(self) {
        STACK_CONTEXT
            .try_with(|cx| *cx.borrow_mut() = Some(self))
            .unwrap();
    }

    fn get() -> Self {
        STACK_CONTEXT
            .try_with(|cx| cx.borrow_mut().take().unwrap())
            .unwrap()
    }

    fn push(expr: impl Into<Cow<'static, str>>) {
        STACK_CONTEXT
            .try_with(|cx| cx.borrow_mut().as_mut().unwrap().exprs.push(expr.into()))
            .unwrap();
    }
}


pub trait Stack: 'static {
    fn stack(&mut self, app: &mut App) -> impl Future<Output = ()> + Send;

    fn name(&self) -> Cow<'static, str> {
        let type_name = std::any::type_name::<Self>();
        Cow::Borrowed(
            type_name
                .split('<')
                .next()
                .unwrap_or(type_name)
                .split("::")
                .last()
                .unwrap_or(type_name),
        )
    }
}
