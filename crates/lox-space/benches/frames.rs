// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Reference-frame rotation benchmarks, organised by branch of the frame tree.
//!
//! They use [`DefaultRotationProvider`], which supplies zero EOP corrections but
//! still evaluates the full precession/nutation/CIP series, so each benchmark
//! reflects the model cost of a branch independent of EOP data acquisition.
//!
//! Groups:
//!
//! * `typed`    — the monomorphised `TryRotation<Icrf, Leaf>` path, one per branch leaf.
//! * `nutation` — ICRF→TOD across every convention, showing the cost spread
//!   between IAU 1980 / 2000A / 2000B / 2006 nutation.
//! * `kernels`  — the raw nutation term-summation per model, in isolation.
//! * `dynamic`  — the same rotations through the `DynFrame` match, showing
//!   dispatch overhead vs. the typed path (the Python-facing path).
//!
//! The whole suite is gated on the `frames` feature; with it off the bench
//! compiles to an empty divan run. Run with `cargo bench -p lox-space --bench frames`.

fn main() {
    divan::main();
}

#[cfg(feature = "frames")]
mod frame_benches {
    use divan::{Bencher, black_box};

    use lox_space::bodies::Earth;
    use lox_space::frames::iers::nutation::Nutation;
    use lox_space::frames::iers::{Iau2000Model, Iers1996, Iers2003, Iers2010, ReferenceSystem};
    use lox_space::frames::providers::DefaultRotationProvider;
    use lox_space::frames::rotations::TryRotation;
    use lox_space::frames::{Cirf, DynFrame, Iau, Icrf, Itrf, Mod, Pef, Teme, Tirf, Tod};
    use lox_space::time::Time;
    use lox_space::time::time_scales::{Tdb, Tt};

    // A fixed epoch inside the definitive EOP range (matches the rotation unit
    // tests). Rotations are time-dependent, so the concrete value only needs to
    // be representative, not special.
    fn epoch() -> Time<Tt> {
        Time::from_two_part_julian_date(Tt, 2454195.5, 0.500754444444444)
    }

    // ---- typed: one full ICRF -> leaf rotation per branch --------------------

    #[divan::bench]
    fn icrf_to_iau_earth(bencher: Bencher) {
        let t = epoch();
        let iau = Iau::new(Earth);
        bencher.bench(|| DefaultRotationProvider.try_rotation(Icrf, iau, black_box(t)));
    }

    // CIO branch: ICRF -> CIRF -> TIRF -> ITRF
    #[divan::bench]
    fn icrf_to_cirf(bencher: Bencher) {
        let t = epoch();
        bencher.bench(|| DefaultRotationProvider.try_rotation(Icrf, Cirf, black_box(t)));
    }

    #[divan::bench]
    fn icrf_to_tirf(bencher: Bencher) {
        let t = epoch();
        bencher.bench(|| DefaultRotationProvider.try_rotation(Icrf, Tirf, black_box(t)));
    }

    #[divan::bench]
    fn icrf_to_itrf(bencher: Bencher) {
        let t = epoch();
        bencher.bench(|| DefaultRotationProvider.try_rotation(Icrf, Itrf, black_box(t)));
    }

    // Equinox branch: ICRF -> MOD -> TOD -> PEF (IERS2003 / IAU 2000A)
    #[divan::bench]
    fn icrf_to_mod(bencher: Bencher) {
        let t = epoch();
        bencher.bench(|| {
            DefaultRotationProvider.try_rotation(Icrf, Mod(Iers2003::default()), black_box(t))
        });
    }

    #[divan::bench]
    fn icrf_to_tod(bencher: Bencher) {
        let t = epoch();
        bencher.bench(|| {
            DefaultRotationProvider.try_rotation(Icrf, Tod(Iers2003::default()), black_box(t))
        });
    }

    #[divan::bench]
    fn icrf_to_pef(bencher: Bencher) {
        let t = epoch();
        bencher.bench(|| {
            DefaultRotationProvider.try_rotation(Icrf, Pef(Iers2003::default()), black_box(t))
        });
    }

    #[divan::bench]
    fn icrf_to_teme(bencher: Bencher) {
        let t = epoch();
        bencher.bench(|| DefaultRotationProvider.try_rotation(Icrf, Teme, black_box(t)));
    }

    // ---- nutation: ICRF -> TOD across conventions ----------------------------
    // Same branch shape, different nutation model, to expose the cost spread.

    #[divan::bench]
    fn tod_iers1996_iau1980(bencher: Bencher) {
        let t = epoch();
        bencher.bench(|| DefaultRotationProvider.try_rotation(Icrf, Tod(Iers1996), black_box(t)));
    }

    #[divan::bench]
    fn tod_iers2003_iau2000a(bencher: Bencher) {
        let t = epoch();
        bencher.bench(|| {
            DefaultRotationProvider.try_rotation(Icrf, Tod(Iers2003(Iau2000Model::A)), black_box(t))
        });
    }

    #[divan::bench]
    fn tod_iers2003_iau2000b(bencher: Bencher) {
        let t = epoch();
        bencher.bench(|| {
            DefaultRotationProvider.try_rotation(Icrf, Tod(Iers2003(Iau2000Model::B)), black_box(t))
        });
    }

    #[divan::bench]
    fn tod_iers2010_iau2006(bencher: Bencher) {
        let t = epoch();
        bencher.bench(|| DefaultRotationProvider.try_rotation(Icrf, Tod(Iers2010), black_box(t)));
    }

    // ---- nutation kernels in isolation ---------------------------------------
    // Raw term-summation cost per model, bypassing the frame-graph composition,
    // to separate kernel cost from the composed ICRF->TOD chains above.

    fn epoch_tdb() -> Time<Tdb> {
        Time::from_two_part_julian_date(Tdb, 2454195.5, 0.500754444444444)
    }

    #[divan::bench]
    fn nutation_iau1980(bencher: Bencher) {
        let t = epoch_tdb();
        bencher.bench(|| Nutation::iau1980(black_box(t)));
    }

    #[divan::bench]
    fn nutation_iau2000a(bencher: Bencher) {
        let t = epoch_tdb();
        bencher.bench(|| Nutation::iau2000a(black_box(t)));
    }

    #[divan::bench]
    fn nutation_iau2000b(bencher: Bencher) {
        let t = epoch_tdb();
        bencher.bench(|| Nutation::iau2000b(black_box(t)));
    }

    #[divan::bench]
    fn nutation_iau2006a(bencher: Bencher) {
        let t = epoch_tdb();
        bencher.bench(|| Nutation::iau2006a(black_box(t)));
    }

    // ---- dynamic: same rotations via the DynFrame match ----------------------
    // Difference vs. the typed group is the dispatch overhead (Python's path).

    #[divan::bench]
    fn dyn_icrf_to_itrf(bencher: Bencher) {
        let t = epoch();
        bencher.bench(|| {
            DefaultRotationProvider.try_rotation(DynFrame::Icrf, DynFrame::Itrf, black_box(t))
        });
    }

    #[divan::bench]
    fn dyn_icrf_to_tod(bencher: Bencher) {
        let t = epoch();
        let target = DynFrame::Tod(ReferenceSystem::Iers2003(Iau2000Model::A));
        bencher
            .bench(|| DefaultRotationProvider.try_rotation(DynFrame::Icrf, target, black_box(t)));
    }

    #[divan::bench]
    fn dyn_icrf_to_iau_earth(bencher: Bencher) {
        let t = epoch();
        let target = DynFrame::Iau(Earth.into());
        bencher
            .bench(|| DefaultRotationProvider.try_rotation(DynFrame::Icrf, target, black_box(t)));
    }

    // The SGP4 hot path: TEME state (dynamic frame) converted to ICRF.
    #[divan::bench]
    fn dyn_teme_to_icrf(bencher: Bencher) {
        let t = epoch();
        bencher.bench(|| {
            DefaultRotationProvider.try_rotation(DynFrame::Teme, DynFrame::Icrf, black_box(t))
        });
    }
}
