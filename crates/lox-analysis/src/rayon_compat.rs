// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

// Sequential fallback for par_iter() when the rayon feature is disabled.
pub(crate) trait FallbackParIter {
    type Item;
    fn par_iter(&self) -> std::slice::Iter<'_, Self::Item>;
}

impl<T> FallbackParIter for Vec<T> {
    type Item = T;
    fn par_iter(&self) -> std::slice::Iter<'_, T> {
        self.iter()
    }
}
