# cdk-rs

Rust support for the [AWS Cloud Development Kit (CDK)](https://aws.amazon.com/cdk/)

```rust
use cdk::{s3, App, Layer, Stack};

struct HelloStack;

impl Stack for HelloStack {
    fn run(me: &mut Layer<Self>) {
        s3::Bucket::new("HelloBucket").stack(me);
    }
}

#[tokio::main]
async fn main() {
    let mut app = App::new().await;
    app.stack(HelloStack).await;
    app.run().await;
}
```

```
cargo cdk build
cargo cdk ls
```
