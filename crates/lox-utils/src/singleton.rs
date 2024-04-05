/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub use lox_derive::Singleton;

pub trait Singleton {
    fn instance() -> Self
    where
        Self: Sized;
}

#[cfg(test)]
mod tests {
    use lox_derive::Singleton;

    use super::*;

    #[test]
    fn test_singleton() {
        #[derive(Singleton, Debug, PartialEq, Eq)]
        struct Test;

        assert_eq!(Test::instance(), Test);
    }
}
