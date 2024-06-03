# cdk-rs

[![License](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/matthunz/cdk-rs#license)
[![Crates.io](https://img.shields.io/crates/v/cargo-cdk.svg)](https://crates.io/crates/cargo-cdk)
[![Crates.io](https://img.shields.io/crates/v/cdk-builder.svg)](https://crates.io/crates/cdk-builder)
[![Docs](https://docs.rs/cdk-builder/badge.svg)](https://docs.rs/cdk-builder/latest/cdk-builder/)
[![CI](https://github.com/matthunz/cdk-rs/workflows/Rust/badge.svg)](https://github.com/matthunz/cdk-rs/actions)

Rust support for the [AWS Cloud Development Kit (CDK)](https://aws.amazon.com/cdk/).

```rust
use cdk::{ec2, s3, App, Layer, Stack};

struct HelloStack;

impl Stack for HelloStack {
    fn run(me: &mut Layer<Self>) {
        s3::Bucket {
            name: "HelloBucket",
            versioned: true,
        }
        .stack(me);

        let vpc = ec2::Vpc {
            name: "HelloVpc",
            max_azs: 3,
        }
        .stack(me);

        ec2::Instance {
            name: "HelloInstance",
            vpc: &vpc,
        }
        .stack(me);
    }
}

#[tokio::main]
async fn main() {
    let mut app = App::new().await;
    app.stack(HelloStack).await;
    app.run().await;
}
```

## Installation

```
cargo install cargo-cdk
```

```
cargo cdk build
cargo cdk ls
```
