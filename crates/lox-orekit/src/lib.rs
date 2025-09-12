use std::{cell::OnceCell, path::Path};

use j4rs::{Instance, InvocationArg, Jvm, JvmBuilder};

#[cfg(test)]
mod test_helpers;
pub mod time;

type JavaResult<T> = j4rs::errors::Result<T>;

pub struct JavaInstance(pub Instance);

impl From<Instance> for JavaInstance {
    fn from(value: Instance) -> Self {
        Self(value)
    }
}

thread_local! {
    static JVM: OnceCell<Jvm> = const { OnceCell::new() };
}

fn with_jvm<F, R>(f: F) -> JavaResult<R>
where
    F: FnOnce(&Jvm) -> JavaResult<R>,
{
    JVM.with(|jvm_cell| {
        // Panic if we do not get a hold of the JVM
        let jvm = jvm_cell.get_or_init(|| JvmBuilder::new().build().unwrap());
        f(jvm)
    })
}

pub fn init<P: AsRef<Path>>(path: P) -> JavaResult<()> {
    with_jvm(|jvm| {
        let path = jvm.create_instance(
            "java.io.File",
            &[&InvocationArg::try_from(path.as_ref().to_str().unwrap())?],
        )?;
        let crawler = jvm.create_instance("org.orekit.data.DirectoryCrawler", &[&path.into()])?;
        let context = jvm.invoke_static(
            "org.orekit.data.DataContext",
            "getDefault",
            InvocationArg::empty(),
        )?;
        let manager = jvm.invoke(&context, "getDataProvidersManager", InvocationArg::empty())?;
        jvm.invoke(&manager, "addProvider", &[&crawler.into()])?;
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use j4rs::InvocationArg;
    use rstest::rstest;

    use crate::test_helpers::init_orekit;
    use crate::with_jvm;

    #[rstest]
    fn test_orekit_smoke(_init_orekit: ()) {
        let name: String = with_jvm(|jvm| {
            let tai = jvm
                .invoke_static(
                    "org.orekit.time.TimeScalesFactory",
                    "getTAI",
                    InvocationArg::empty(),
                )
                .unwrap();
            let name = jvm.invoke(&tai, "getName", InvocationArg::empty()).unwrap();
            jvm.to_rust(name)
        })
        .unwrap();
        assert_eq!(name, "TAI".to_owned())
    }
}
