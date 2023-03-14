pub mod barycenters;
pub mod minor;
pub mod planets;
pub mod satellites;

pub trait NaifId {
    fn id() -> i32;
}

pub fn naif_id<T: NaifId>(_: T) -> i32 {
    <T as NaifId>::id()
}

pub trait Ellipsoid {
    fn max_equatorial_radius() -> f64;
    fn min_equatorial_radius() -> f64;
    fn polar_radius() -> f64;
    fn mean_radius() -> f64;
}

pub fn mean_radius<T: Ellipsoid>(_: T) -> f64 {
    <T as Ellipsoid>::mean_radius()
}

pub struct Sun;

impl NaifId for Sun {
    fn id() -> i32 {
        10
    }
}

#[cfg(test)]
mod tests {
    use super::planets::Earth;
    use super::*;

    #[test]
    fn test_mean_radius() {
        assert_eq!(mean_radius(Earth), 6371.008366666666);
    }
}
