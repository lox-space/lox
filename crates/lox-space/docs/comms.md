<!--
SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>

SPDX-License-Identifier: MPL-2.0
-->

# Communications

RF link budget analysis for space communication systems.

## Modulation Schemes

| Name | Bits per Symbol |
|------|-----------------|
| `BPSK` | 1 |
| `QPSK` | 2 |
| `8PSK` | 3 |
| `16QAM` | 4 |
| `32QAM` | 5 |
| `64QAM` | 6 |
| `128QAM` | 7 |
| `256QAM` | 8 |

## Antenna Patterns

| Pattern | Description |
|---------|-------------|
| `ParabolicPattern` | Airy disk model for parabolic reflector antennas |
| `GaussianPattern` | Gaussian roll-off approximation |
| `DipolePattern` | Short and general dipole radiation patterns |

Pattern gain is evaluated at two angles: `theta`, the polar angle from
boresight, and `phi`, the azimuth about boresight measured from the
antenna-frame `+X` axis toward `+Y`. All built-in patterns are axially
symmetric and ignore `phi`, so it can be omitted:

```python
import lox_space as lox

pattern = lox.ParabolicPattern(diameter=0.98 * lox.m, efficiency=0.45)
gain = pattern.gain(29 * lox.GHz, theta=1.0 * lox.deg)
```

## Antenna Frames

An `AntennaFrame` orients an antenna pattern in a parent frame as a
right-handed orthonormal basis: `+Z` is the boresight (`theta = 0`) and `+X`
is the `phi = 0` reference direction. Construct it from a boresight and a
reference direction, or use `AntennaFrame.identity()` for a frame aligned
with the parent frame (boresight along `+Z`):

```python
import lox_space as lox

frame = lox.AntennaFrame.from_boresight_and_reference(
    boresight=[1.0, 0.0, 0.0], reference=[0.0, 0.0, 1.0]
)
theta, phi = frame.angles_for([0.0, 0.0, 1.0])
```

A `PatternedAntenna` combines a pattern with a frame (identity by default)
and can evaluate its gain directly toward a parent-frame direction vector
via `gain_toward`:

```python
pattern = lox.ParabolicPattern(diameter=0.98 * lox.m, efficiency=0.45)
antenna = lox.PatternedAntenna(pattern=pattern, frame=frame)
gain = antenna.gain_toward(29 * lox.GHz, [1.0, 0.0, 0.0])  # on boresight
```

## Hardware Inventory: CommsPayload

A `CommsPayload` models the communications hardware of one platform as
inventory plus wiring: antennas, radios, and lumped models are added to the
inventory, ports wire an antenna to a radio (with a per-path feed loss), and
terminals expose the operational endpoints that link analysis addresses.
Shared hardware is expressed naturally — a diplexer is two ports referencing
the same antenna:

```python
import lox_space as lox

ka_band = lox.FrequencyRange(27.0 * lox.GHz, 31.0 * lox.GHz)

payload = lox.CommsPayload()
dish = payload.add_antenna("dish", lox.ConstantAntenna(gain=46.0 * lox.dB))
pa = payload.add_transmitter(
    "pa",
    lox.AmplifierTransmitter(band=ka_band, power=10 * lox.W),
)
lnb = payload.add_receiver(
    "lnb",
    lox.NoiseTempReceiver(band=ka_band, noise_temperature=500 * lox.K),
)
tx_port = payload.add_tx_port("tx leg", dish, pa, 1.0 * lox.dB, band=ka_band)
rx_port = payload.add_rx_port(
    "rx leg", dish, lnb, 0.5 * lox.dB, antenna_noise_temperature=0.0 * lox.K
)
terminal = payload.add_transceiver_terminal("ka transceiver", tx_port=tx_port, rx_port=rx_port)
```

For the common single-chain cases the one-shot constructors return the
payload and its terminal directly:

```python
tx_payload, tx_terminal = lox.CommsPayload.eirp_only("tx", ka_band, 55.0 * lox.dB)
rx_payload, rx_terminal = lox.CommsPayload.gt_only("rx", ka_band, 3.01 * lox.dB)
```

`LinkStats.for_link` computes a link budget between two terminals. The
carrier is a link-level input and must lie inside both endpoints' supported
frequency ranges:

```python
link = lox.LinkStats.for_link(
    tx_payload,
    tx_terminal,
    rx_payload,
    rx_terminal,
    carrier=29.0 * lox.GHz,
    bandwidth=5.0 * lox.MHz,
    range=1000.0 * lox.km,
    direction="downlink",
)
print(f"C/N0 = {float(link.c_n0):.2f} dB·Hz")
```

Payloads attach to assets for scenario analysis via
`GroundStation(..., comms_payload=...)` and `Spacecraft(..., comms_payload=...)`.

## Lumped EIRP and G/T

For early-phase mission studies — where manufacturer datasheets typically
publish only aggregate figures — you can build a link budget directly from
lumped EIRP and G/T models. Use `CommsPayload.eirp_only` and
`CommsPayload.gt_only`:

```python
import lox_space as lox

ka_band = lox.FrequencyRange(27.0 * lox.GHz, 31.0 * lox.GHz)
tx_payload, tx_terminal = lox.CommsPayload.eirp_only("tx", ka_band, 55.0 * lox.dB)
rx_payload, rx_terminal = lox.CommsPayload.gt_only("rx", ka_band, 3.01 * lox.dB)
link = lox.LinkStats.for_link(
    tx_payload,
    tx_terminal,
    rx_payload,
    rx_terminal,
    carrier=29.0 * lox.GHz,
    bandwidth=5.0 * lox.MHz,
    range=1000.0 * lox.km,
    direction="downlink",
)
print(f"C/N0 = {float(link.c_n0):.2f} dB·Hz")
```

For lumped links, `link.carrier_rx_power` and `link.noise_power` are `None` —
the absolute carrier and noise power are not recoverable from EIRP and G/T
alone. The carrier-to-noise density ratio (`c_n0`) and carrier-to-noise ratio
(`c_n`) remain available.

To compute modulation-aware figures (`Es/N0`, `Eb/N0`, link margin), apply a
`Channel`:

```python
channel = lox.Channel(
    link_type="downlink",
    symbol_rate=5 * lox.MHz,
    required_eb_n0=10.0 * lox.dB,
    margin=3.0 * lox.dB,
    modulation=lox.Modulation("QPSK"),
    roll_off=0.35,
    fec=0.5,
)
modulated = channel.apply(link)
print(f"Margin = {modulated.margin.as_float():.2f} dB")
```

Use the component tier (configure antennas, amplifiers, receiver noise) when
you need the full breakdown — for example for noise-budget allocation or
detailed component trade studies.

## Quick Example

```python
import lox_space as lox

# Define a Ka-band downlink
frequency = 29 * lox.GHz

band = lox.FrequencyRange(27.0 * lox.GHz, 31.0 * lox.GHz)

# Transmitter: satellite with parabolic antenna, 1 dB feed loss
tx_pattern = lox.ParabolicPattern(diameter=0.98 * lox.m, efficiency=0.45)
tx_antenna = lox.PatternedAntenna(pattern=tx_pattern)
tx_payload, tx_terminal = lox.CommsPayload.transmitter_only(
    "satellite", tx_antenna, lox.AmplifierTransmitter(band=band, power=10 * lox.W),
    feed_loss=1.0 * lox.dB,
)

# Receiver: ground station with known system noise temperature
rx_antenna = lox.ConstantAntenna(gain=40.0 * lox.dB)
rx_payload, rx_terminal = lox.CommsPayload.receiver_only(
    "ground station", rx_antenna,
    lox.NoiseTempReceiver(band=band, noise_temperature=200 * lox.K),
    feed_loss=0.0 * lox.dB, antenna_noise_temperature=0.0 * lox.K,
)

# Define a QPSK channel at 5 Msps
channel = lox.Channel(
    link_type="downlink",
    symbol_rate=5 * lox.MHz,
    required_eb_n0=10.0 * lox.dB,
    margin=3.0 * lox.dB,
    modulation=lox.Modulation("QPSK"),
    roll_off=0.35,
    fec=0.5,
)

# Compute a full link budget at 1000 km slant range.
# Pointing defaults to boresight; pass tx_angle/rx_angle for off-boresight
# angles or tx_direction/rx_direction for line-of-sight vectors.
link = lox.LinkStats.for_link(
    tx_payload,
    tx_terminal,
    rx_payload,
    rx_terminal,
    carrier=frequency,
    bandwidth=channel.bandwidth(),
    range=1000 * lox.km,
    direction="downlink",
)
modulated = channel.apply(link)

print(f"EIRP:        {float(link.eirp):.1f} dBW")
print(f"FSPL:        {float(link.fspl):.1f} dB")
print(f"C/N0:        {float(link.c_n0):.1f} dB·Hz")
print(f"Es/N0:       {float(modulated.es_n0):.1f} dB")
print(f"Eb/N0:       {float(modulated.eb_n0):.1f} dB")
print(f"Link margin: {float(modulated.margin):.1f} dB")
```

### Direction-Aware Pointing

For patterned antennas the link budget can derive the pattern angles directly
from a line-of-sight vector expressed in the antenna's parent frame, using the
antenna's `AntennaFrame`. The derived angles are reported on the result:

```python
import lox_space as lox

# Dish boresight along +X
frame = lox.AntennaFrame(boresight=[1.0, 0.0, 0.0], reference=[0.0, 0.0, 1.0])
pattern = lox.ParabolicPattern(diameter=0.98 * lox.m, efficiency=0.45)
antenna = lox.PatternedAntenna(pattern=pattern, frame=frame)
tx_payload, tx_terminal = lox.CommsPayload.transmitter_only(
    "satellite", antenna,
    lox.AmplifierTransmitter(band=band, power=10.0 * lox.W),
    feed_loss=1.0 * lox.dB,
)

link = lox.LinkStats.for_link(
    tx_payload,
    tx_terminal,
    rx_payload,
    rx_terminal,
    carrier=29 * lox.GHz,
    bandwidth=channel.bandwidth(),
    range=1000 * lox.km,
    direction="downlink",
    tx_direction=[0.9, 0.1, 0.0],  # line of sight in the TX parent frame
)
print(f"TX theta: {link.tx_theta.to_degrees():.2f} deg")
print(f"TX phi:   {link.tx_phi.to_degrees():.2f} deg")
```

### Working with Decibels

```python
import lox_space as lox

# Create from dB value or linear ratio
gain = 30.0 * lox.dB
gain_linear = lox.Decibel.from_linear(1000.0)

# Arithmetic
total = gain + 3.0 * lox.dB   # 33.0 dB
diff = gain - 10.0 * lox.dB   # 20.0 dB

# Convert back
print(f"{float(gain)} dB = {gain.to_linear():.0f} linear")
```

### Free-Space Path Loss

```python
import lox_space as lox

# FSPL at 1000 km range and 29 GHz
loss = lox.fspl(distance=1000 * lox.km, frequency=29 * lox.GHz)
print(f"FSPL: {float(loss):.1f} dB")
```

### Environmental Losses

```python
import lox_space as lox

losses = lox.EnvironmentalLosses(
    rain=2.0 * lox.dB,
    gaseous=0.3 * lox.dB,
    atmospheric=0.5 * lox.dB,
)
print(f"Total: {float(losses.total()):.1f} dB")

# Pass to LinkStats.for_link via the losses parameter
link = lox.LinkStats.for_link(
    tx_payload,
    tx_terminal,
    rx_payload,
    rx_terminal,
    carrier=29 * lox.GHz,
    bandwidth=channel.bandwidth(),
    range=1000 * lox.km,
    direction="downlink",
    losses=losses,
)
```

---

::: lox_space.Decibel
    options:
      show_source: false

---

::: lox_space.Modulation
    options:
      show_source: false

---

::: lox_space.ParabolicPattern
    options:
      show_source: false

---

::: lox_space.GaussianPattern
    options:
      show_source: false

---

::: lox_space.DipolePattern
    options:
      show_source: false

---

::: lox_space.ConstantAntenna
    options:
      show_source: false

---

::: lox_space.AntennaFrame
    options:
      show_source: false

---

::: lox_space.PatternedAntenna
    options:
      show_source: false

---

::: lox_space.AmplifierTransmitter
    options:
      show_source: false

---

::: lox_space.NoiseTempReceiver
    options:
      show_source: false

---

::: lox_space.CascadeReceiver
    options:
      show_source: false

---

::: lox_space.NoiseStage
    options:
      show_source: false

---

::: lox_space.Channel
    options:
      show_source: false

---

::: lox_space.EnvironmentalLosses
    options:
      show_source: false

---

::: lox_space.LinkStats
    options:
      show_source: false

---

::: lox_space.ModulatedLinkStats
    options:
      show_source: false

---

::: lox_space.fspl
    options:
      show_source: false

---

::: lox_space.freq_overlap
    options:
      show_source: false

---

::: lox_space.power_flux_density
    options:
      show_source: false

---

::: lox_space.PfdMask
    options:
      show_source: false

---

::: lox_space.FrequencyRange
    options:
      show_source: false

---

::: lox_space.CommsPayload
    options:
      show_source: false

---

::: lox_space.slant_range
    options:
      show_source: false
