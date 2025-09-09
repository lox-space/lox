use std::error::Error;

use j4rs::{JvmBuilder, MavenArtifact};

const OREKIT_VERSION: &str = "13.1";
const HIPPARCHUS_VERSION: &str = "4.0.1";

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo::rerun-if-changed=build.rs");

    let jvm = JvmBuilder::new().build()?;

    let artifacts: [String; _] = [
        format!("org.orekit:orekit:{}", OREKIT_VERSION),
        format!("org.hipparchus:hipparchus-core:{}", HIPPARCHUS_VERSION),
        format!("org.hipparchus:hipparchus-geometry:{}", HIPPARCHUS_VERSION),
        format!("org.hipparchus:hipparchus-ode:{}", HIPPARCHUS_VERSION),
        format!("org.hipparchus:hipparchus-fitting:{}", HIPPARCHUS_VERSION),
        format!("org.hipparchus:hipparchus-optim:{}", HIPPARCHUS_VERSION),
        format!("org.hipparchus:hipparchus-filtering:{}", HIPPARCHUS_VERSION),
        format!("org.hipparchus:hipparchus-stat:{}", HIPPARCHUS_VERSION),
    ];

    artifacts
        .into_iter()
        .map(MavenArtifact::from)
        .try_for_each(|a| jvm.deploy_artifact(&a))?;

    Ok(())
}
