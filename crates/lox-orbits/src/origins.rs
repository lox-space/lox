use lox_bodies::Origin;

pub trait Origin {}

impl<U: Origin> Origin for U {}

pub trait CoordinateOrigin<T: Origin> {
    fn origin(&self) -> T;
}
