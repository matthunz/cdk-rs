use crate::{Layer, Stack};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Bucket<'a> {
    pub name: &'a str,
    pub versioned: bool,
}

impl Stack for Bucket<'_> {
    fn run(_me: &mut Layer<Self>) {}

    fn initialize(me: &mut Layer<Self>) {
        me.parent_exprs.borrow_mut().push(format!(
            r#"
                new s3.Bucket(this, '{}', {{
                    versioned: {}
                }});
            "#,
            &me.name, me.versioned
        ));
    }
}
