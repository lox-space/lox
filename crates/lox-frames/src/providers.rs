use lox_time::OffsetProvider;

#[macro_export]
macro_rules! transform_provider {
    ($provider:ident) => {
        impl $crate::transformations::TransformProvider for $provider {}
    };
}

#[derive(Copy, Clone, Debug, OffsetProvider)]
pub struct DefaultTransformProvider;

transform_provider!(DefaultTransformProvider);
