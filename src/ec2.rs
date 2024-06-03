use crate::{Layer, Stack};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Vpc<'a> {
    pub name: &'a str,
    pub max_azs: u32,
}

impl Stack for Vpc<'_> {
    fn run(_me: &mut Layer<Self>) {}

    fn initialize(me: &mut Layer<Self>) {
        me.parent_exprs.borrow_mut().push(format!(
            r#"
                new ec2.Vpc(this, '{}', {{
                    maxAzs: {}
                }});
            "#,
            me.name, me.max_azs,
        ));
    }
}
