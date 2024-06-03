use crate::{Layer, Stack};

pub struct Bucket {
    name: String,
}

impl Bucket {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

impl Stack for Bucket {
    fn run(me: &mut Layer<Self>) {
        me.exprs.borrow_mut().push(format!(
            r#"
                new s3.Bucket(this, '{}', {{
                    versioned: true
                }});
            "#,
            &me.name
        ));
    }
}
