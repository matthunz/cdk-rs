use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use std::borrow::Cow;
use std::cell::RefCell;
use std::ops::Deref;
use std::ops::DerefMut;
use std::process::Stdio;
use std::rc::Rc;
use std::sync::atomic::AtomicU64;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{ChildStdin, ChildStdout, Command};

pub mod ec2;

pub mod s3;

#[derive(Serialize)]
struct Request<'a> {
    js: &'a str,
}

#[derive(Deserialize)]
struct Response {
    json: Value,
}

#[derive(Clone)]
pub struct App {
    exprs: Rc<RefCell<Vec<String>>>,
}

impl App {
    pub fn new() -> Self {
        Self {
            exprs: Rc::default(),
        }
    }

    pub fn stack<S: Stack>(&mut self, stack: S) {
        let mut layer = Layer {
            app: self.clone(),
            stack,
            exprs: Rc::default(),
            parent_exprs: self.exprs.clone(),
            expr: None,
        };
        S::run(&mut layer);
    }

    pub async fn run(&mut self) {
        let mut child = Command::new("node")
            .arg("worker.js")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed to start child process");

        let mut stdin = child.stdin.take().unwrap();
        let mut stdout = child.stdout.take().unwrap();

        tokio::task::spawn(async move {
            child.wait().await.unwrap();
        });

        request::<i32>(
            r#"
                app = new cdk.App();
                0
            "#,
            &mut stdin,
            &mut stdout,
        )
        .await;

        let exprs = self.exprs.borrow();
        for expr in &*exprs {
            request::<Value>(expr, &mut stdin, &mut stdout).await;
        }
    }
}

async fn request<T>(js: &str, stdin: &mut ChildStdin, stdout: &mut ChildStdout) -> T
where
    T: for<'de> Deserialize<'de>,
{
    let message = Request { js };
    let mut bytes = serde_json::to_vec(&message).unwrap();
    bytes.push(b'\n');
    stdin.write_all(&bytes).await.unwrap();

    let reader = BufReader::new(stdout);
    let mut lines = reader.lines();

    let Ok(Some(line)) = lines.next_line().await else {
        todo!()
    };

    let res: Response = serde_json::from_str(&line).unwrap();
    serde_json::from_value(res.json).unwrap()
}

pub trait Stack: Sized {
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

    fn setup(me: &mut Layer<Self>) {
        let _ = me;
    }

    fn initialize(me: &mut Layer<Self>) {
        let exprs = me.exprs.borrow().concat();
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
            me.stack.name()
        );
        me.parent_exprs.borrow_mut().push(js)
    }

    fn stack<T: Stack>(self, layer: &Layer<T>) -> Layer<Self> {
        let mut layer = Layer {
            app: layer.app.clone(),
            stack: self,
            exprs: Rc::default(),
            parent_exprs: layer.exprs.clone(),
            expr: None,
        };
        Self::setup(&mut layer);
        layer
    }
}

pub struct Layer<T: Stack> {
    app: App,
    stack: T,
    exprs: Rc<RefCell<Vec<String>>>,
    expr: Option<String>,
    parent_exprs: Rc<RefCell<Vec<String>>>,
}

impl<T: Stack> Deref for Layer<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.stack
    }
}

impl<T: Stack> DerefMut for Layer<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.stack
    }
}

impl<T: Stack> Drop for Layer<T> {
    fn drop(&mut self) {
        T::initialize(self)
    }
}

static COUNT: AtomicU64 = AtomicU64::new(0);
