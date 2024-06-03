use crate::{Layer, Stack};
use std::borrow::Cow;

pub struct Bucket {
    name: Cow<'static, str>,
}

impl Bucket {
    pub fn new(name: impl Into<Cow<'static, str>>) -> Self {
        Self { name: name.into() }
    }
}

impl Stack for Bucket {
    fn run(_me: &mut Layer<Self>) {}

    fn initialize(me: &mut Layer<Self>) {
        me.parent_exprs.borrow_mut().push(format!(
            r#"
                new s3.Bucket(this, '{}', {{
                    versioned: true
                }});
            "#,
            &me.name
        ));
    }
}
