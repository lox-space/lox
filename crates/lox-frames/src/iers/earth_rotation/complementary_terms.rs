// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

pub struct ComplimentaryTerm {
    pub nfa: [i64; 8],
    pub s: f64,
    pub c: f64,
}

pub const E0: [ComplimentaryTerm; 33] = [
    ComplimentaryTerm {
        nfa: [0, 0, 0, 0, 1, 0, 0, 0],
        s: 2640.96e-6,
        c: -0.39e-6,
    },
    ComplimentaryTerm {
        nfa: [0, 0, 0, 0, 2, 0, 0, 0],
        s: 63.52e-6,
        c: -0.02e-6,
    },
    ComplimentaryTerm {
        nfa: [0, 0, 2, -2, 3, 0, 0, 0],
        s: 11.75e-6,
        c: 0.01e-6,
    },
    ComplimentaryTerm {
        nfa: [0, 0, 2, -2, 1, 0, 0, 0],
        s: 11.21e-6,
        c: 0.01e-6,
    },
    ComplimentaryTerm {
        nfa: [0, 0, 2, -2, 2, 0, 0, 0],
        s: -4.55e-6,
        c: 0.00e-6,
    },
    ComplimentaryTerm {
        nfa: [0, 0, 2, 0, 3, 0, 0, 0],
        s: 2.02e-6,
        c: 0.00e-6,
    },
    ComplimentaryTerm {
        nfa: [0, 0, 2, 0, 1, 0, 0, 0],
        s: 1.98e-6,
        c: 0.00e-6,
    },
    ComplimentaryTerm {
        nfa: [0, 0, 0, 0, 3, 0, 0, 0],
        s: -1.72e-6,
        c: 0.00e-6,
    },
    ComplimentaryTerm {
        nfa: [0, 1, 0, 0, 1, 0, 0, 0],
        s: -1.41e-6,
        c: -0.01e-6,
    },
    ComplimentaryTerm {
        nfa: [0, 1, 0, 0, -1, 0, 0, 0],
        s: -1.26e-6,
        c: -0.01e-6,
    },
    /* 11-20 */
    ComplimentaryTerm {
        nfa: [1, 0, 0, 0, -1, 0, 0, 0],
        s: -0.63e-6,
        c: 0.00e-6,
    },
    ComplimentaryTerm {
        nfa: [1, 0, 0, 0, 1, 0, 0, 0],
        s: -0.63e-6,
        c: 0.00e-6,
    },
    ComplimentaryTerm {
        nfa: [0, 1, 2, -2, 3, 0, 0, 0],
        s: 0.46e-6,
        c: 0.00e-6,
    },
    ComplimentaryTerm {
        nfa: [0, 1, 2, -2, 1, 0, 0, 0],
        s: 0.45e-6,
        c: 0.00e-6,
    },
    ComplimentaryTerm {
        nfa: [0, 0, 4, -4, 4, 0, 0, 0],
        s: 0.36e-6,
        c: 0.00e-6,
    },
    ComplimentaryTerm {
        nfa: [0, 0, 1, -1, 1, -8, 12, 0],
        s: -0.24e-6,
        c: -0.12e-6,
    },
    ComplimentaryTerm {
        nfa: [0, 0, 2, 0, 0, 0, 0, 0],
        s: 0.32e-6,
        c: 0.00e-6,
    },
    ComplimentaryTerm {
        nfa: [0, 0, 2, 0, 2, 0, 0, 0],
        s: 0.28e-6,
        c: 0.00e-6,
    },
    ComplimentaryTerm {
        nfa: [1, 0, 2, 0, 3, 0, 0, 0],
        s: 0.27e-6,
        c: 0.00e-6,
    },
    ComplimentaryTerm {
        nfa: [1, 0, 2, 0, 1, 0, 0, 0],
        s: 0.26e-6,
        c: 0.00e-6,
    },
    /* 21-30 */
    ComplimentaryTerm {
        nfa: [0, 0, 2, -2, 0, 0, 0, 0],
        s: -0.21e-6,
        c: 0.00e-6,
    },
    ComplimentaryTerm {
        nfa: [0, 1, -2, 2, -3, 0, 0, 0],
        s: 0.19e-6,
        c: 0.00e-6,
    },
    ComplimentaryTerm {
        nfa: [0, 1, -2, 2, -1, 0, 0, 0],
        s: 0.18e-6,
        c: 0.00e-6,
    },
    ComplimentaryTerm {
        nfa: [0, 0, 0, 0, 0, 8, -13, -1],
        s: -0.10e-6,
        c: 0.05e-6,
    },
    ComplimentaryTerm {
        nfa: [0, 0, 0, 2, 0, 0, 0, 0],
        s: 0.15e-6,
        c: 0.00e-6,
    },
    ComplimentaryTerm {
        nfa: [2, 0, -2, 0, -1, 0, 0, 0],
        s: -0.14e-6,
        c: 0.00e-6,
    },
    ComplimentaryTerm {
        nfa: [1, 0, 0, -2, 1, 0, 0, 0],
        s: 0.14e-6,
        c: 0.00e-6,
    },
    ComplimentaryTerm {
        nfa: [0, 1, 2, -2, 2, 0, 0, 0],
        s: -0.14e-6,
        c: 0.00e-6,
    },
    ComplimentaryTerm {
        nfa: [1, 0, 0, -2, -1, 0, 0, 0],
        s: 0.14e-6,
        c: 0.00e-6,
    },
    ComplimentaryTerm {
        nfa: [0, 0, 4, -2, 4, 0, 0, 0],
        s: 0.13e-6,
        c: 0.00e-6,
    },
    /* 31-33 */
    ComplimentaryTerm {
        nfa: [0, 0, 2, -2, 4, 0, 0, 0],
        s: -0.11e-6,
        c: 0.00e-6,
    },
    ComplimentaryTerm {
        nfa: [1, 0, -2, 0, -3, 0, 0, 0],
        s: 0.11e-6,
        c: 0.00e-6,
    },
    ComplimentaryTerm {
        nfa: [1, 0, -2, 0, -1, 0, 0, 0],
        s: 0.11e-6,
        c: 0.00e-6,
    },
];

pub const E1: [ComplimentaryTerm; 1] = [ComplimentaryTerm {
    nfa: [0, 0, 0, 0, 1, 0, 0, 0],
    s: -0.87e-6,
    c: 0.00e-6,
}];
