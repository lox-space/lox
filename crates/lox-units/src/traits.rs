use thiserror::Error;

#[derive(Debug, Error)]
#[error("no dynamic type available")]
pub struct DynError;

pub trait ToDyn {
    type Output;

    fn to_dyn(&self) -> Self::Output;
}
