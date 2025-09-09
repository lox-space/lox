use std::{cell::OnceCell, convert::Infallible, path::Path};

use j4rs::{InvocationArg, Jvm, JvmBuilder};
use lox_time::time_scales::{TimeScale, offsets::TryOffset};

thread_local! {
    static JVM: OnceCell<Jvm> = const { OnceCell::new() };
}

fn with_jvm<F, R>(f: F) -> j4rs::errors::Result<R>
where
    F: FnOnce(&Jvm) -> j4rs::errors::Result<R>,
{
    JVM.with(|jvm_cell| {
        // Panic if we do not get a hold of the JVM
        let jvm = jvm_cell.get_or_init(|| JvmBuilder::new().build().unwrap());
        f(jvm)
    })
}

pub fn init<P: AsRef<Path>>(path: P) -> j4rs::errors::Result<()> {
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

pub struct OrekitOffsetProvider;

impl<T: TimeScale> TryOffset<T, T> for OrekitOffsetProvider {
    type Error = Infallible;

    fn try_offset(
        &self,
        _origin: T,
        _target: T,
        _delta: lox_time::deltas::TimeDelta,
    ) -> Result<lox_time::deltas::TimeDelta, Self::Error> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use j4rs::InvocationArg;
    use rstest::{fixture, rstest};

    use crate::with_jvm;

    #[fixture]
    fn init_orekit() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("tests/data");
        super::init(path).unwrap()
    }

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
