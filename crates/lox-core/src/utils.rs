// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

pub struct Linspace {
    start: f64,
    end: f64,
    n: usize,
    current: usize,
}

impl Linspace {
    pub fn new(start: f64, end: f64, n: usize) -> Self {
        Self {
            start,
            end,
            n,
            current: 0,
        }
    }
}

impl Iterator for Linspace {
    type Item = f64;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= self.n {
            return None;
        }

        let value = if self.n == 1 {
            self.start
        } else {
            let t = self.current as f64 / (self.n - 1) as f64;
            self.start + t * (self.end - self.start)
        };

        self.current += 1;
        Some(value)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.n - self.current;
        (remaining, Some(remaining))
    }
}

impl ExactSizeIterator for Linspace {}
