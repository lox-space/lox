use lox_time::offset_provider;

#[macro_export]
macro_rules! transform_provider {
    ($provider:ident) => {
        impl $crate::transformations::TransformProvider for $provider {}
    };
}

#[derive(Copy, Clone, Debug)]
pub struct DefaultTransformProvider;

offset_provider!(DefaultTransformProvider);
transform_provider!(DefaultTransformProvider);
