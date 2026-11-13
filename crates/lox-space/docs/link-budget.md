<!--
SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
SPDX-FileCopyrightText: 2026 Torkil Rein Gustavsen <torkilrg@ksat.no>

SPDX-License-Identifier: MPL-2.0
-->

# Link Budget Analysis

[Visibility analysis](events.md) answers whether two assets can see each
other. A link budget analysis refines those windows down to the intervals in
which a radio link between a pair of terminals actually *closes* — where the
link margin is non-negative — and annotates each surviving window with the
throughput it supports.

## Terminals pair up automatically

Ground stations and spacecraft carry named terminals (see
[Communications](comms.md)). Every transmit terminal is matched against every
receive terminal on the other asset whose band contains the carrier, in both
directions, so a single visible pair can yield several links:

| Pair | Links | Direction |
| --- | --- | --- |
| ground ↔ space | spacecraft transmit → station receive | `"downlink"` |
| | station transmit → spacecraft receive | `"uplink"` |
| space ↔ space | either spacecraft's transmit → the other's receive | `"crosslink"` |

Terminals model hardware and know nothing about symbol rates, so the carrier,
waveform and coding come from a `CarrierPlan` held by the analysis. Pass
`overrides` to give a particular terminal pair a plan of its own.

Results are keyed by `TerminalId`, which pairs an asset identifier with the
terminal's name on that asset — a terminal name alone is only unique within
its asset.

## What "closes" means

A `CarrierPlan` carries a table of candidate MODCODs and a *detection* MODCOD,
which defaults to the most robust entry in the table. The window boundaries are
the roots of the margin against that single detection MODCOD; adaptive coding
and modulation then annotates each window with the highest-efficiency mode that
closes at its peak.

The split matters. Selecting the best mode at every sample would make the
margin jump back to near zero each time the selection climbs a rung, leaving
nothing for a root finder to bracket. Detecting against one fixed mode keeps
the margin smooth and unimodal — it peaks at closest approach, where both path
loss and atmospheric loss are smallest.

## Propagation losses

Losses carried on the plan are applied to every sample. They default to none,
so a plan without them is a clear-sky analysis. To model the atmosphere, build
a `PropagationLosses` — by hand, or from the ITU-R models in
[Atmospheric Propagation](itur.md) — and attach it with the `losses` argument.

Note that ITU-R attenuation is quoted against an exceedance probability, so a
plan built that way answers "when does the link close at that availability?"
rather than "when does the link close right now".

### Losses that follow the elevation

A fixed loss is one figure for the whole pass, yet atmospheric loss grows
steeply towards the horizon: at 5° elevation it is roughly five times the loss
at 30°. To let the loss follow the spacecraft along the pass, describe each
ground station's site with a `SitePropagation` and hand it to the analysis
through `site_propagation`:

```python
itu = lox.ItuProvider("lox-itur-data.npz")


def site_for(station: lox.GroundStation) -> lox.SitePropagation:
    location = station.location()
    return itu.site_propagation(
        lat=location.latitude(),
        lon=location.longitude(),
        carriers=[8.2 * lox.GHz, 7.2 * lox.GHz],  # downlink, uplink
        probability=5.0,
        diameter=3.0 * lox.m,
    )


results = lox.LinkBudgetAnalysis(
    scenario, visibility, plan, site_propagation=site_for
).compute()
```

The callable is invoked once per ground station of the scenario; return `None`
to leave a station out. To cover individual stations instead, pass a dict of
ground-station ids to `SitePropagation`.

A `SitePropagation` holds one `ElevationProfile` per carrier — atmospheric
losses tabulated over a grid of elevations and interpolated in between. Each ground-space link
looks its profile up by the carrier of its plan, within 1 %, so the uplink and
the downlink each find their own. Then:

- The profile **replaces the atmospheric lines** of the plan's losses — rain,
  gaseous, cloud and scintillation — and keeps the plan's other lines, such
  as a pointing margin. This applies to
  overridden terminal pairs as well.
- On a downlink the receive noise temperature degrades with the absorptive
  part of the loss, so G/T follows the elevation too.
- A link whose carrier no profile describes **keeps the plan's losses**, and
  so does every inter-satellite link. `LinkWindow.loss_source` reports which
  was used — `"site_profile"` or `"plan"` — so a silent fallback is visible.

Site propagation is opt-in: without the argument every link uses the plan's
fixed losses.

The exceedance probability, the antenna diameter and the polarisation are part
of the profile. To compare availabilities, build one `SitePropagation` per
probability and run the analysis once for each.

!!! note
    The ITU-R models are valid from 5° elevation upwards and hold their 5°
    value below it, and so does a profile built from them. For a station
    tracking below 5° the loss near the horizon is underestimated.

## Pointing

Both ends are evaluated at antenna boresight, which is the right model for a
tracking dish and for a nadir-pointed spacecraft antenna. Off-boresight loss
needs spacecraft attitude, which a scenario does not yet model.

## Quick Example

```python
import lox_space as lox

# A spacecraft that downlinks in X band, and a station that receives it.
sat = lox.Spacecraft(
    "sat-7",
    sgp4,
    tx_terminals={"downlink": lox.EirpModel("X", 6.0 * lox.dB)},
)
station = lox.GroundStation(
    "Kiruna",
    lox.EllipsoidLocation(
        "IAU_EARTH", 20.96688 * lox.deg, 67.858428 * lox.deg, 0 * lox.km
    ),
    min_elevation=5 * lox.deg,
    rx_terminals={"main": lox.GtModel("X", 16.0 * lox.dB)},
)

scenario = lox.Scenario(t0, t1, spacecraft=[sat], ground_stations=[station])
visibility = lox.VisibilityAnalysis(scenario).compute()

plan = lox.CarrierPlan(
    8.2 * lox.GHz,
    lox.Channel(20.0 * lox.MHz),
    lox.ModCod.dvb_s2(),
    design_margin=3.0 * lox.dB,
)
results = lox.LinkBudgetAnalysis(scenario, visibility, plan).compute()

tx = lox.TerminalId("sat-7", "downlink")
rx = lox.TerminalId("Kiruna", "main")
for window in results.windows(tx, rx):
    print(
        f"{window.interval.start()}  "
        f"peak {window.peak_margin.to_decibels():.1f} dB  "
        f"{window.peak_modcod.name}  "
        f"{window.data_volume.to(lox.GB):.2f} GB"
    )

print(f"Total downlinked: {results.data_volume(tx, rx).to(lox.GB):.1f} GB")
```

---

::: lox_space.TerminalId
    options:
      show_source: false

---

::: lox_space.CarrierPlan
    options:
      show_source: false

---

::: lox_space.LinkBudgetAnalysis
    options:
      show_source: false

---

::: lox_space.LinkBudgetResults
    options:
      show_source: false

---

::: lox_space.LinkWindow
    options:
      show_source: false
