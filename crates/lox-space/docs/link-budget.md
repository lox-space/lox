<!--
SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>

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
        f"peak {float(window.peak_margin):.1f} dB  "
        f"{window.peak_modcod.name}  "
        f"{window.data_volume / 8e9:.2f} GB"
    )

print(f"Total downlinked: {results.data_volume(tx, rx) / 8e9:.1f} GB")
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
