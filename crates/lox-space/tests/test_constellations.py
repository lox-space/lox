# SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
#
# SPDX-License-Identifier: MPL-2.0

import pytest
import lox_space as lox


@pytest.fixture
def epoch():
    return lox.Time("TAI", 2025, 1, 1)


class TestWalkerDelta:
    def test_basic(self, epoch):
        c = lox.Constellation.walker_delta(
            "iridium",
            epoch,
            lox.Origin("Earth"),
            nsats=66,
            nplanes=6,
            semi_major_axis=7159 * lox.km,
            inclination=53 * lox.deg,
            phasing=1,
        )
        assert len(c) == 66
        assert c.name == "iridium"

    def test_plane_distribution(self, epoch):
        c = lox.Constellation.walker_delta(
            "test",
            epoch,
            lox.Origin("Earth"),
            nsats=12,
            nplanes=3,
            semi_major_axis=7000 * lox.km,
            inclination=53 * lox.deg,
        )
        planes = {s.plane for s in c.satellites}
        assert planes == {0, 1, 2}
        for plane in range(3):
            count = sum(1 for s in c.satellites if s.plane == plane)
            assert count == 4

    def test_repr(self, epoch):
        c = lox.Constellation.walker_delta(
            "test",
            epoch,
            lox.Origin("Earth"),
            nsats=6,
            nplanes=3,
            semi_major_axis=7000 * lox.km,
            inclination=53 * lox.deg,
        )
        assert "test" in repr(c)
        assert "6" in repr(c)

    def test_mismatch_error(self, epoch):
        with pytest.raises(ValueError, match="not divisible"):
            lox.Constellation.walker_delta(
                "bad",
                epoch,
                lox.Origin("Earth"),
                nsats=7,
                nplanes=3,
                semi_major_axis=7000 * lox.km,
                inclination=53 * lox.deg,
            )


class TestWalkerStar:
    def test_basic(self, epoch):
        c = lox.Constellation.walker_star(
            "polar",
            epoch,
            lox.Origin("Earth"),
            nsats=8,
            nplanes=4,
            semi_major_axis=7000 * lox.km,
            inclination=90 * lox.deg,
        )
        assert len(c) == 8
        assert c.name == "polar"


class TestStreetOfCoverage:
    def test_basic(self, epoch):
        c = lox.Constellation.street_of_coverage(
            "soc",
            epoch,
            lox.Origin("Earth"),
            nsats=24,
            nplanes=4,
            semi_major_axis=7159 * lox.km,
            inclination=53 * lox.deg,
            coverage_fold=1,
        )
        assert len(c) == 24
        assert c.name == "soc"

    def test_constraint_error(self, epoch):
        with pytest.raises(ValueError, match="coverage fold"):
            lox.Constellation.street_of_coverage(
                "bad",
                epoch,
                lox.Origin("Earth"),
                nsats=8,
                nplanes=4,
                semi_major_axis=7159 * lox.km,
                inclination=53 * lox.deg,
                coverage_fold=1,
            )

    def test_equatorial_error(self, epoch):
        with pytest.raises(ValueError, match="non-equatorial"):
            lox.Constellation.street_of_coverage(
                "bad",
                epoch,
                lox.Origin("Earth"),
                nsats=24,
                nplanes=4,
                semi_major_axis=7159 * lox.km,
                inclination=0 * lox.deg,
                coverage_fold=1,
            )


class TestFlower:
    def test_with_perigee_altitude(self, epoch):
        c = lox.Constellation.flower(
            "flower14",
            epoch,
            lox.Origin("Earth"),
            n_petals=14,
            n_days=1,
            nsats=28,
            phasing_numerator=1,
            phasing_denominator=28,
            inclination=53 * lox.deg,
            perigee_altitude=780 * lox.km,
        )
        assert len(c) == 28
        assert c.name == "flower14"
        # All in plane 0
        assert all(s.plane == 0 for s in c.satellites)

    def test_with_sma(self, epoch):
        c = lox.Constellation.flower(
            "flower",
            epoch,
            lox.Origin("Earth"),
            n_petals=14,
            n_days=1,
            nsats=5,
            phasing_numerator=1,
            phasing_denominator=28,
            inclination=53 * lox.deg,
            semi_major_axis=7000 * lox.km,
            eccentricity=0.01,
        )
        assert len(c) == 5

    def test_missing_shape(self, epoch):
        with pytest.raises(ValueError):
            lox.Constellation.flower(
                "bad",
                epoch,
                lox.Origin("Earth"),
                n_petals=14,
                n_days=1,
                nsats=5,
                phasing_numerator=1,
                phasing_denominator=28,
                inclination=53 * lox.deg,
            )


class TestConstellationSatellite:
    def test_properties(self, epoch):
        c = lox.Constellation.walker_delta(
            "test",
            epoch,
            lox.Origin("Earth"),
            nsats=6,
            nplanes=3,
            semi_major_axis=7000 * lox.km,
            inclination=53 * lox.deg,
        )
        sat = c.satellites[0]
        assert sat.plane == 0
        assert sat.index_in_plane == 0
        assert "plane=0" in repr(sat)


class TestFlowerEdgeCases:
    def test_mutually_exclusive_error(self, epoch):
        with pytest.raises(ValueError, match="mutually exclusive"):
            lox.Constellation.flower(
                "bad",
                epoch,
                lox.Origin("Earth"),
                n_petals=14,
                n_days=1,
                nsats=5,
                phasing_numerator=1,
                phasing_denominator=28,
                inclination=53 * lox.deg,
                perigee_altitude=780 * lox.km,
                semi_major_axis=7000 * lox.km,
                eccentricity=0.01,
            )

    def test_missing_eccentricity_error(self, epoch):
        with pytest.raises(ValueError, match="eccentricity"):
            lox.Constellation.flower(
                "bad",
                epoch,
                lox.Origin("Earth"),
                n_petals=14,
                n_days=1,
                nsats=5,
                phasing_numerator=1,
                phasing_denominator=28,
                inclination=53 * lox.deg,
                semi_major_axis=7000 * lox.km,
            )

    def test_with_argument_of_periapsis(self, epoch):
        c = lox.Constellation.flower(
            "flower_aop",
            epoch,
            lox.Origin("Earth"),
            n_petals=14,
            n_days=1,
            nsats=5,
            phasing_numerator=1,
            phasing_denominator=28,
            inclination=53 * lox.deg,
            semi_major_axis=7000 * lox.km,
            eccentricity=0.01,
            argument_of_periapsis=45 * lox.deg,
        )
        assert len(c) == 5


class TestWalkerStarEdgeCases:
    def test_with_phasing(self, epoch):
        c = lox.Constellation.walker_star(
            "phased",
            epoch,
            lox.Origin("Earth"),
            nsats=8,
            nplanes=4,
            semi_major_axis=7000 * lox.km,
            inclination=90 * lox.deg,
            phasing=2,
        )
        assert len(c) == 8

    def test_with_argument_of_periapsis(self, epoch):
        c = lox.Constellation.walker_star(
            "aop",
            epoch,
            lox.Origin("Earth"),
            nsats=8,
            nplanes=4,
            semi_major_axis=7000 * lox.km,
            inclination=90 * lox.deg,
            argument_of_periapsis=90 * lox.deg,
        )
        assert len(c) == 8


class TestWalkerDeltaEdgeCases:
    def test_with_eccentricity(self, epoch):
        c = lox.Constellation.walker_delta(
            "ecc",
            epoch,
            lox.Origin("Earth"),
            nsats=6,
            nplanes=3,
            semi_major_axis=7000 * lox.km,
            inclination=53 * lox.deg,
            eccentricity=0.01,
        )
        assert len(c) == 6

    def test_with_argument_of_periapsis(self, epoch):
        c = lox.Constellation.walker_delta(
            "aop",
            epoch,
            lox.Origin("Earth"),
            nsats=6,
            nplanes=3,
            semi_major_axis=7000 * lox.km,
            inclination=53 * lox.deg,
            argument_of_periapsis=45 * lox.deg,
        )
        assert len(c) == 6


class TestStreetOfCoverageEdgeCases:
    def test_with_argument_of_periapsis(self, epoch):
        c = lox.Constellation.street_of_coverage(
            "soc_aop",
            epoch,
            lox.Origin("Earth"),
            nsats=24,
            nplanes=4,
            semi_major_axis=7159 * lox.km,
            inclination=53 * lox.deg,
            argument_of_periapsis=30 * lox.deg,
        )
        assert len(c) == 24


class TestPropagatorOption:
    def test_j2_propagator(self, epoch):
        c = lox.Constellation.walker_delta(
            "j2test",
            epoch,
            lox.Origin("Earth"),
            nsats=6,
            nplanes=3,
            semi_major_axis=7000 * lox.km,
            inclination=53 * lox.deg,
            propagator="j2",
        )
        assert len(c) == 6

    def test_invalid_propagator(self, epoch):
        with pytest.raises(ValueError, match="unknown propagator"):
            lox.Constellation.walker_delta(
                "bad",
                epoch,
                lox.Origin("Earth"),
                nsats=6,
                nplanes=3,
                semi_major_axis=7000 * lox.km,
                inclination=53 * lox.deg,
                propagator="invalid",
            )
