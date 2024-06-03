# cdk-rs

Rust support for the [AWS Cloud Development Kit (CDK)](https://aws.amazon.com/cdk/)

```rust
use cdk::{s3, App, Stack};

struct HelloStack;

impl Stack for HelloStack {
    async fn stack(&mut self, _app: &mut App) {
        s3::Bucket::new("HelloBucket");
    }
}

#[tokio::main]
async fn main() {
    let mut app = App::new().await;
    app.stack(HelloStack).await;
}
```

```
cargo cdk build
cargo cdk ls
```
