use lox_math::types::julian_dates::Epoch;

#[cfg(feature = "python")]
pub mod python;
pub mod spk;

pub(crate) type Position = (f64, f64, f64);
pub(crate) type Velocity = (f64, f64, f64);
pub(crate) type Body = i32;

pub trait Ephemeris {
    type Error: std::error::Error;

    fn position(&self, epoch: Epoch, origin: Body, target: Body) -> Result<Position, Self::Error>;
    fn velocity(&self, epoch: Epoch, origin: Body, target: Body) -> Result<Velocity, Self::Error>;
    fn state(
        &self,
        epoch: Epoch,
        origin: Body,
        target: Body,
    ) -> Result<(Position, Velocity), Self::Error>;
}

fn ancestors(id: i32) -> Vec<i32> {
    let mut ancestors = vec![id];
    let mut current = id;
    while current != 0 {
        current /= 100;
        ancestors.push(current);
    }
    ancestors
}

pub fn path_from_ids(origin: i32, target: i32) -> Vec<i32> {
    let ancestors_origin = ancestors(origin);
    let ancestors_target = ancestors(target);
    let n = ancestors_target.len();
    let mut path = ancestors_origin;

    ancestors_target
        .into_iter()
        .take(n - 1)
        .rev()
        .for_each(|id| path.push(id));

    if *path.first().unwrap() != 0 && *path.last().unwrap() != 0 {
        let idx = path.iter().position(|&id| id == 0).unwrap();
        if path[idx - 1] == path[idx + 1] {
            let common_ancestor = vec![path[idx - 1]];
            path.splice((idx - 1)..=(idx + 1), common_ancestor);
        }
    }

    path
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ancestors() {
        assert_eq!(ancestors(0), vec![0]);
        assert_eq!(ancestors(3), vec![3, 0]);
        assert_eq!(ancestors(399), vec![399, 3, 0]);
    }

    #[test]
    fn test_path_from_ids() {
        assert_eq!(path_from_ids(399, 499), [399, 3, 0, 4, 499]);
        assert_eq!(path_from_ids(399, 0), [399, 3, 0]);
        assert_eq!(path_from_ids(0, 399), [0, 3, 399]);
        assert_eq!(path_from_ids(399, 3), [399, 3]);
        assert_eq!(path_from_ids(3, 399), [3, 399]);
        assert_eq!(path_from_ids(399, 301), [399, 3, 301]);
    }
}
