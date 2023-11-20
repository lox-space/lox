/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::marker::PhantomData;

/// Marker types indicating the conventions followed by the body.
struct IERS2003;
struct MHB2000;

struct Moon<C> {
    /// Phantom data has no runtime representation, so the struct is still zero-sized.
    _convention: PhantomData<C>,
}

trait MeanLongitudeOfAscendingNode {
    fn mean_longitude_of_ascending_node(&self) -> f64;
}

/// We can then implement the same trait for different marker types.
impl MeanLongitudeOfAscendingNode for Moon<IERS2003> {
    fn mean_longitude_of_ascending_node(&self) -> f64 {
        0.0
    }
}

impl MeanLongitudeOfAscendingNode for Moon<MHB2000> {
    fn mean_longitude_of_ascending_node(&self) -> f64 {
        0.0
    }
}

/// And it's still possible to implement methods that don't care about the conventions
/// followed by the body.
impl Body for Moon<T> {
    fn naif_id() -> i32 {
        301
    }

    fn name() -> &'static str {
        "Moon"
    }
}

fn example_usage() {
    let iers2003_mean_long = Moon::<IERS2003>.mean_longitude_of_ascending_node();
    let mhb2000_mean_long = Moon::<MHB2000>.mean_longitude_of_ascending_node();
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_example_usage_compiles() {
        super::example_usage();
    }
}
