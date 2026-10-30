# SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
#
# SPDX-License-Identifier: MPL-2.0

import pytest

import lox_space as lox

# ---------------------------------------------------------------------------
# Fixtures
# ---------------------------------------------------------------------------

X_BAND = "X"
CARRIER = 8.2 * lox.GHz
# Sized so the link closes around the middle of a pass but not at the horizon,
# which is what makes the refinement assertions below non-vacuous.
SYMBOL_RATE = 20.0 * lox.MHz


@pytest.fixture(scope="module")
def oneweb_subset(oneweb):
    """Three OneWeb satellites — enough to exercise pairing, fast to run."""
    return dict(list(oneweb.items())[:3])


@pytest.fixture(scope="module")
def ground_assets(estrack):
    """Two stations, each with a modest X-band receive terminal."""
    return [
        lox.GroundStation(
            gs.id(),
            gs.location(),
            min_elevation=gs.min_elevation(),
            horizon_mask=gs.horizon_mask(),
            rx_terminals={"main": lox.GtModel(X_BAND, 16.0 * lox.dB)},
        )
        for gs in estrack[:2]
    ]


@pytest.fixture(scope="module")
def space_assets(oneweb_subset):
    """Spacecraft with a low-power X-band downlink terminal."""
    return [
        lox.Spacecraft(
            name,
            sgp4,
            tx_terminals={"downlink": lox.EirpModel(X_BAND, 6.0 * lox.dB)},
        )
        for name, sgp4 in oneweb_subset.items()
    ]


@pytest.fixture(scope="module")
def t0(oneweb_subset):
    return next(iter(oneweb_subset.values())).time()


@pytest.fixture(scope="module")
def scenario(t0, space_assets, ground_assets):
    return lox.Scenario(
        t0,
        t0 + lox.TimeDelta(86400),
        spacecraft=space_assets,
        ground_stations=ground_assets,
    )


@pytest.fixture(scope="module")
def visibility(scenario):
    return lox.VisibilityAnalysis(scenario).compute()


@pytest.fixture(scope="module")
def plan():
    return lox.CarrierPlan(
        CARRIER,
        lox.Channel(SYMBOL_RATE),
        lox.ModCod.dvb_s2(),
    )


@pytest.fixture(scope="module")
def results(scenario, visibility, plan):
    return lox.LinkBudgetAnalysis(scenario, visibility, plan).compute()


def ids(space_assets, ground_assets):
    return (
        lox.TerminalId(space_assets[0].id(), "downlink"),
        lox.TerminalId(ground_assets[0].id(), "main"),
    )


# ---------------------------------------------------------------------------
# TerminalId
# ---------------------------------------------------------------------------


class TestTerminalId:
    def test_accessors(self):
        tid = lox.TerminalId("sat-7", "downlink")
        assert tid.asset == "sat-7"
        assert tid.name == "downlink"
        assert str(tid) == "sat-7/downlink"
        assert repr(tid) == 'TerminalId("sat-7", "downlink")'

    def test_equality_and_hash(self):
        a = lox.TerminalId("sat-7", "downlink")
        b = lox.TerminalId("sat-7", "downlink")
        c = lox.TerminalId("sat-7", "beacon")
        assert a == b
        assert hash(a) == hash(b)
        assert a != c
        assert len({a, b, c}) == 2


# ---------------------------------------------------------------------------
# CarrierPlan
# ---------------------------------------------------------------------------


class TestCarrierPlan:
    def test_defaults(self, plan):
        assert plan.carrier.to_gigahertz() == pytest.approx(8.2)
        assert float(plan.design_margin) == pytest.approx(0.0)
        assert len(plan.modcods) == len(lox.ModCod.dvb_s2())

    def test_detection_modcod_is_most_robust(self, plan):
        weakest = min(float(mc.required_es_n0) for mc in lox.ModCod.dvb_s2())
        assert float(plan.detection_modcod.required_es_n0) == pytest.approx(weakest)

    def test_empty_modcod_table_is_rejected(self):
        with pytest.raises(ValueError, match="at least one MODCOD"):
            lox.CarrierPlan(CARRIER, lox.Channel(SYMBOL_RATE), [])

    def test_explicit_detection_modcod(self):
        qpsk = next(mc for mc in lox.ModCod.dvb_s2() if mc.name == "QPSK 3/4")
        plan = lox.CarrierPlan(
            CARRIER,
            lox.Channel(SYMBOL_RATE),
            lox.ModCod.dvb_s2(),
            detection_modcod=qpsk,
        )
        assert plan.detection_modcod.name == "QPSK 3/4"

    def test_detection_modcod_outside_the_table_is_rejected(self):
        table = lox.ModCod.dvb_s2()
        outsider = next(mc for mc in table if mc.name == "QPSK 3/4")
        others = [mc for mc in table if mc.name != "QPSK 3/4"]
        with pytest.raises(ValueError, match="not in the carrier plan"):
            lox.CarrierPlan(
                CARRIER,
                lox.Channel(SYMBOL_RATE),
                others,
                detection_modcod=outsider,
            )

    def test_repr(self, plan):
        assert repr(plan).startswith("CarrierPlan(8.200 GHz")


# ---------------------------------------------------------------------------
# LinkBudgetAnalysis
# ---------------------------------------------------------------------------


class TestLinkBudgetAnalysis:
    @pytest.mark.parametrize("seconds", [0, -60])
    def test_non_positive_step_is_rejected(self, scenario, visibility, plan, seconds):
        analysis = lox.LinkBudgetAnalysis(
            scenario, visibility, plan, step=lox.TimeDelta(seconds)
        )
        with pytest.raises(ValueError, match="step must be positive"):
            analysis.compute()

    def test_links_close(self, results):
        assert not results.is_empty()
        assert results.num_links() > 0
        assert results.total_windows() > 0

    def test_repr(self, scenario, visibility, plan):
        analysis = lox.LinkBudgetAnalysis(scenario, visibility, plan)
        assert repr(analysis) == (
            "LinkBudgetAnalysis(2 ground stations, 3 spacecraft, 0 overrides)"
        )

    def test_carrier_outside_band_yields_no_links(self, scenario, visibility):
        s_band = lox.CarrierPlan(
            2.2 * lox.GHz, lox.Channel(SYMBOL_RATE), lox.ModCod.dvb_s2()
        )
        results = lox.LinkBudgetAnalysis(scenario, visibility, s_band).compute()
        assert results.is_empty()

    def test_design_margin_can_close_nothing(self, scenario, visibility):
        impossible = lox.CarrierPlan(
            CARRIER,
            lox.Channel(SYMBOL_RATE),
            lox.ModCod.dvb_s2(),
            design_margin=1000.0 * lox.dB,
        )
        results = lox.LinkBudgetAnalysis(scenario, visibility, impossible).compute()
        assert results.is_empty()
        assert results.num_links() == 0

    def test_override_applies_to_one_pair_only(
        self, scenario, visibility, plan, space_assets, ground_assets
    ):
        tx, rx = ids(space_assets, ground_assets)
        impossible = lox.CarrierPlan(
            CARRIER,
            lox.Channel(SYMBOL_RATE),
            lox.ModCod.dvb_s2(),
            design_margin=1000.0 * lox.dB,
        )
        baseline = lox.LinkBudgetAnalysis(scenario, visibility, plan).compute()
        assert baseline.windows(tx, rx), "the overridden pair must close by default"

        results = lox.LinkBudgetAnalysis(
            scenario, visibility, plan, overrides=[(tx, rx, impossible)]
        ).compute()

        assert results.windows(tx, rx) == []
        # Every other link is untouched.
        assert results.num_links() == baseline.num_links() - 1

    def test_explicit_ensemble_matches_auto_propagation(
        self, scenario, visibility, plan
    ):
        auto = lox.LinkBudgetAnalysis(scenario, visibility, plan).compute()
        explicit = lox.LinkBudgetAnalysis(
            scenario, visibility, plan, ensemble=scenario.propagate()
        ).compute()
        assert auto.total_windows() == explicit.total_windows()


# ---------------------------------------------------------------------------
# LinkBudgetResults
# ---------------------------------------------------------------------------


class TestLinkBudgetResults:
    def test_refinement_actually_shrinks_the_windows(self, results, visibility):
        """The fit-out must be marginal, or the subset test below proves nothing."""
        visible = sum(
            float(iv.duration())
            for intervals in visibility.all_intervals().values()
            for iv in intervals
        )
        closed = sum(
            float(w.interval.duration())
            for windows in results.all_windows().values()
            for w in windows
        )
        assert 0.0 < closed < visible, (
            f"expected partial closure, got {closed}s of {visible}s visible"
        )

    def test_windows_are_subsets_of_visibility(
        self, results, visibility, space_assets, ground_assets
    ):
        for tx, rx in results.link_ids():
            windows = results.windows(tx, rx)
            intervals = visibility.intervals(
                tx.asset, rx.asset
            ) or visibility.intervals(rx.asset, tx.asset)
            assert intervals, "every link must come from a visible pair"
            for window in windows:
                assert any(
                    window.interval.start() >= iv.start()
                    and window.interval.end() <= iv.end()
                    for iv in intervals
                ), "a link window escaped its visibility window"

    def test_window_fields(self, results, space_assets, ground_assets):
        # An explicit pair, not `link_ids()[0]`: that order comes from a hash
        # map and picks a different link on every run.
        tx, rx = ids(space_assets, ground_assets)
        window = results.windows(tx, rx)[0]
        assert window.direction == "downlink"
        assert float(window.peak_margin) >= float(window.min_margin)
        assert window.data_volume > 0.0
        assert window.peak_information_rate.to_hertz() > 0.0
        assert window.peak_modcod is not None
        assert repr(window).startswith("LinkWindow(downlink")

    def test_data_volume_sums_windows(self, results, space_assets, ground_assets):
        tx, rx = ids(space_assets, ground_assets)
        total = sum(w.data_volume for w in results.windows(tx, rx))
        assert results.data_volume(tx, rx) == pytest.approx(total)

    def test_all_windows_keys(self, results):
        all_windows = results.all_windows()
        assert len(all_windows) == results.num_links()
        for tx, rx in all_windows:
            assert "/" in tx and "/" in rx

    def test_unknown_terminals_return_empty(self, results):
        tx = lox.TerminalId("nonexistent", "downlink")
        rx = lox.TerminalId("nowhere", "main")
        assert results.windows(tx, rx) == []
        assert results.data_volume(tx, rx) == 0.0

    def test_repr(self, results):
        assert repr(results).startswith("LinkBudgetResults(")
