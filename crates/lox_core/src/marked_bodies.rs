/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::bodies::{Body, NaifId};
use std::marker::PhantomData;

/// Problem: different conventions define the same fundamental arguments differently for the same
/// bodies. For example, the mean longitude of the ascending node of the Moon is defined differently
/// in the IERS2003 and MHB2000 conventions. The set of traits defined in fundamental.rs is
/// IERS2003-only, but we don't want to define a new trait for each combination of conventions and
/// argument, which will quickly get out of hand and lead to long, convoluted method names to
/// avoid clashes when implemented on the same body. Nor do we want to define a new body for
/// each set of conventions – there are enough of them as it is.
///
/// Proposed solution: Following a recommendation in Rust for Rustaceans, we can define a marker type for
/// each convention, make the body generic over the marker, and implement the same trait for each
/// marker. The body remains zero-sized, its instances are constant, and there's no need to
/// namespace fundamental arguments trait declarations.
///
/// This is also clearer for consumers of the API, who no longer have to wonder why mod fundamental
/// provides only IERS03 fundamental args.

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
impl<T> Body for Moon<T> {
    fn id(&self) -> NaifId {
        NaifId(301)
    }

    fn name(&self) -> &'static str {
        "Moon"
    }
}

/// One disadvantage is that our ZSTs can no longer be used as types or instances interchangeably,
/// but, still being zero-sized, these objects are considered constant and can be defined as such.
const MOON_IERS2003: Moon<IERS2003> = Moon {
    _convention: PhantomData,
};

const MOON_MHB2000: Moon<MHB2000> = Moon {
    _convention: PhantomData,
};

fn example_usage() {
    let iers2003_mean_long = MOON_IERS2003.mean_longitude_of_ascending_node();
    let mhb2000_mean_long = MOON_MHB2000.mean_longitude_of_ascending_node();
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_example_usage_compiles() {
        super::example_usage();
    }
}
