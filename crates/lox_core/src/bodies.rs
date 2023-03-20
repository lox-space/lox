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
    fn polar_radius() -> f64;
    fn mean_radius() -> f64;
}

pub fn polar_radius<T: Ellipsoid>(_: T) -> f64 {
    <T as Ellipsoid>::polar_radius()
}

pub fn mean_radius<T: Ellipsoid>(_: T) -> f64 {
    <T as Ellipsoid>::mean_radius()
}

pub trait Spheroid: Ellipsoid {
    fn equatorial_radius() -> f64;
}

pub fn equatorial_radius<T: Spheroid>(_: T) -> f64 {
    <T as Spheroid>::equatorial_radius()
}

pub trait TriAxial: Ellipsoid {
    fn subplanetary_radius() -> f64;
    fn along_orbit_radius() -> f64;
}

pub fn subplanetary_radius<T: TriAxial>(_: T) -> f64 {
    <T as TriAxial>::subplanetary_radius()
}

pub fn along_orbit_radius<T: TriAxial>(_: T) -> f64 {
    <T as TriAxial>::along_orbit_radius()
}

pub trait PointMass {
    fn gravitational_parameter() -> f64;
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
    fn test_naif_id() {
        assert_eq!(naif_id(Sun), 10);
    }

    #[test]
    fn test_mean_radius() {
        assert_eq!(mean_radius(Earth), 6371.008366666666);
    }
}
