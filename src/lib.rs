use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use std::borrow::Cow;
use std::cell::RefCell;
use std::ops::Deref;
use std::process::Stdio;
use std::rc::Rc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{ChildStdin, ChildStdout, Command};
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

struct AppInner {
    stdin: ChildStdin,
    stdout: ChildStdout,
    handle: JoinHandle<()>,
}

#[derive(Clone)]
pub struct App {
    inner: Rc<RefCell<AppInner>>,
    exprs: Rc<RefCell<Vec<String>>>,
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

        let me = App {
            inner: Rc::new(RefCell::new(AppInner {
                stdin,
                stdout,
                handle,
            })),
            exprs: Rc::default(),
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

    pub async fn stack<S: Stack>(&mut self, stack: S) {
        let mut layer = Layer {
            app: self.clone(),
            stack,
            exprs: Rc::default(),
            parent_exprs: self.exprs.clone(),
        };
        S::run(&mut layer);
    }

    async fn request<T>(&self, js: &str) -> T
    where
        T: for<'de> Deserialize<'de>,
    {
        let mut me = self.inner.borrow_mut();

        let message = Request { js };
        let mut bytes = serde_json::to_vec(&message).unwrap();
        bytes.push(b'\n');
        me.stdin.write_all(&bytes).await.unwrap();

        let reader = BufReader::new(&mut me.stdout);
        let mut lines = reader.lines();

        let Ok(Some(line)) = lines.next_line().await else {
            todo!()
        };

        let res: Response = serde_json::from_str(&line).unwrap();
        serde_json::from_value(res.json).unwrap()
    }

    pub async fn run(&mut self) {
        let exprs = self.exprs.borrow();

        for expr in &*exprs {
            self.request::<Value>(expr).await;
        }
    }
}

impl Drop for App {
    fn drop(&mut self) {
        self.inner.borrow_mut().handle.abort();
    }
}

pub trait Stack: Sized + 'static {
    fn run(me: &mut Layer<Self>);

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

    fn stack<T: Stack>(self, layer: &Layer<T>) -> Layer<Self> {
        Layer {
            app: layer.app.clone(),
            stack: self,
            exprs: Rc::default(),
            parent_exprs: layer.exprs.clone(),
        }
    }
}

pub struct Layer<T: Stack> {
    app: App,
    stack: T,
    exprs: Rc<RefCell<Vec<String>>>,
    parent_exprs: Rc<RefCell<Vec<String>>>,
}

impl<T: Stack> Deref for Layer<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        todo!()
    }
}

impl<T: Stack> Drop for Layer<T> {
    fn drop(&mut self) {
        let exprs = self.exprs.borrow().concat();
        let js = format!(
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
            self.stack.name()
        );
        self.parent_exprs.borrow_mut().push(js)
    }
}
