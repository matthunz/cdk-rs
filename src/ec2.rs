use std::sync::atomic::Ordering;

use crate::{Layer, Stack, COUNT};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Vpc<'a> {
    pub name: &'a str,
    pub max_azs: u32,
}

impl Stack for Vpc<'_> {
    fn run(_me: &mut Layer<Self>) {}

    fn setup(me: &mut Layer<Self>) {
        let id = COUNT.fetch_add(1, Ordering::SeqCst);

        let expr = format!(
            r#"
            if (stacks['{id}'] == null) {{
                stacks['{id}'] = new ec2.Vpc(this, '{}', {{
                    maxAzs: {}
                }});
            }}

            return stacks['{id}']
        "#,
            me.name, me.max_azs,
        );
        me.expr = Some(expr.clone());
    }

    fn initialize(me: &mut Layer<Self>) {
        me.exprs
            .borrow_mut()
            .push(me.expr.as_ref().unwrap().clone());
    }
}

#[derive(Clone)]
pub struct Instance<'a> {
    pub name: &'a str,
    pub vpc: &'a Layer<Vpc<'a>>,
}

impl Stack for Instance<'_> {
    fn run(_me: &mut Layer<Self>) {}

    fn initialize(me: &mut Layer<Self>) {
        let id = COUNT.fetch_add(1, Ordering::SeqCst);
        let vpc = me.vpc.expr.as_ref().unwrap().clone();

        me.parent_exprs.borrow_mut().push(format!(
            r#"
                if (stacks['{id}'] == null) {{
                    stacks['{id}'] = new ec2.Instance(this, '{}', {{
                        instanceType: ec2.InstanceType.of(ec2.InstanceClass.BURSTABLE2, ec2.InstanceSize.MICRO),
                        machineImage: ec2.MachineImage.latestAmazonLinux2(),
                        vpc: (() => {{ {vpc} }})()
                    }});
                }}

                return stacks['{id}']
            "#,
            me.name,
        ));
    }
}
