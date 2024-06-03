use crate::StackContext;

pub struct Bucket {}

impl Bucket {
    pub fn new(name: &str) {
        StackContext::push(format!(
            r#"
                new s3.Bucket(this, '{}', {{
                    versioned: true
                }});
            "#,
            name
        ));
    }
}
