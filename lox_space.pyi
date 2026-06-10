# SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
#
# SPDX-License-Identifier: MPL-2.0

from collections.abc import Callable
from typing import Literal, Self, overload
import numpy as np
from numpy.typing import ArrayLike
import os

type Scale = Literal["TAI", "TCB", "TCG", "TDB", "TT", "UT1"]
type Epoch = Literal["jd", "mjd", "j1950", "j2000"]
type Unit = Literal["seconds", "days", "centuries"]

# Exceptions
class NonFiniteTimeDeltaError(Exception):
    """Raised when a TimeDelta operation produces a non-finite result."""

    ...

# Unit classes
class Angle:
    """Angle type for type-safe angular values.

    Use with unit constants: `45 * lox.deg` or `1.5 * lox.rad`
    Convert to float with `float(angle)` (returns radians).
    """
    def __new__(cls, value: float) -> Self: ...
    def __add__(self, other: Angle) -> Angle: ...
    def __sub__(self, other: Angle) -> Angle: ...
    def __neg__(self) -> Angle: ...
    def __mul__(self, other: float) -> Angle: ...
    def __rmul__(self, other: float) -> Angle: ...
    def __eq__(self, other: object) -> bool: ...
    def __repr__(self) -> str: ...
    def __str__(self) -> str: ...
    def __complex__(self) -> complex: ...
    def __float__(self) -> float: ...
    def __int__(self) -> int: ...
    def to_radians(self) -> float:
        """Returns the value in radians."""
        ...
    def to_degrees(self) -> float:
        """Returns the value in degrees."""
        ...
    def to_arcseconds(self) -> float:
        """Returns the value in arcseconds."""
        ...

class AngularRate:
    """Angular rate type for type-safe angular velocity values.

    Use with unit constants: `1.0 * lox.rad_per_s` or `15 * lox.deg_per_s`
    Convert to float with `float(rate)` (returns rad/s).
    """
    def __new__(cls, value: float) -> Self: ...
    def __add__(self, other: AngularRate) -> AngularRate: ...
    def __sub__(self, other: AngularRate) -> AngularRate: ...
    def __neg__(self) -> AngularRate: ...
    def __mul__(self, other: float) -> AngularRate: ...
    def __rmul__(self, other: float) -> AngularRate: ...
    def __eq__(self, other: object) -> bool: ...
    def __repr__(self) -> str: ...
    def __str__(self) -> str: ...
    def __complex__(self) -> complex: ...
    def __float__(self) -> float: ...
    def __int__(self) -> int: ...
    def to_radians_per_second(self) -> float:
        """Returns the value in radians per second."""
        ...
    def to_degrees_per_second(self) -> float:
        """Returns the value in degrees per second."""
        ...

class Distance:
    """Distance type for type-safe length values.

    Use with unit constants: `100 * lox.km` or `1.5 * lox.au`
    Convert to float with `float(distance)` (returns meters).
    """
    def __new__(cls, value: float) -> Self: ...
    def __add__(self, other: Distance) -> Distance: ...
    def __sub__(self, other: Distance) -> Distance: ...
    def __neg__(self) -> Distance: ...
    def __mul__(self, other: float) -> Distance: ...
    def __rmul__(self, other: float) -> Distance: ...
    def __eq__(self, other: object) -> bool: ...
    def __repr__(self) -> str: ...
    def __str__(self) -> str: ...
    def __complex__(self) -> complex: ...
    def __float__(self) -> float: ...
    def __int__(self) -> int: ...
    def to_meters(self) -> float:
        """Returns the value in meters."""
        ...
    def to_kilometers(self) -> float:
        """Returns the value in kilometers."""
        ...
    def to_astronomical_units(self) -> float:
        """Returns the value in astronomical units."""
        ...

class Frequency:
    """Frequency type for type-safe frequency values.

    Use with unit constants: `2.4 * lox.GHz` or `100 * lox.MHz`
    Convert to float with `float(frequency)` (returns Hz).
    """
    def __new__(cls, value: float) -> Self: ...
    def __add__(self, other: Frequency) -> Frequency: ...
    def __sub__(self, other: Frequency) -> Frequency: ...
    def __neg__(self) -> Frequency: ...
    def __mul__(self, other: float) -> Frequency: ...
    def __rmul__(self, other: float) -> Frequency: ...
    def __eq__(self, other: object) -> bool: ...
    def __repr__(self) -> str: ...
    def __str__(self) -> str: ...
    def __complex__(self) -> complex: ...
    def __float__(self) -> float: ...
    def __int__(self) -> int: ...
    def to_hertz(self) -> float:
        """Returns the value in hertz."""
        ...
    def to_kilohertz(self) -> float:
        """Returns the value in kilohertz."""
        ...
    def to_megahertz(self) -> float:
        """Returns the value in megahertz."""
        ...
    def to_gigahertz(self) -> float:
        """Returns the value in gigahertz."""
        ...
    def to_terahertz(self) -> float:
        """Returns the value in terahertz."""
        ...

class GravitationalParameter:
    """Gravitational parameter (GM) type.

    Constructor takes the value in m³/s².
    Convert to float with `float(gm)` (returns m³/s²).
    """
    def __new__(cls, value: float) -> Self: ...
    @staticmethod
    def from_km3_per_s2(value: float) -> GravitationalParameter:
        """Creates from a value in km³/s²."""
        ...
    def to_m3_per_s2(self) -> float:
        """Returns the value in m³/s²."""
        ...
    def to_km3_per_s2(self) -> float:
        """Returns the value in km³/s²."""
        ...
    def __float__(self) -> float: ...
    def __eq__(self, other: object) -> bool: ...
    def __repr__(self) -> str: ...
    def __str__(self) -> str: ...

class Power:
    """Power type for type-safe power values.

    Use with unit constants: `10 * lox.W` or `1.5 * lox.kW`
    Convert to float with `float(power)` (returns Watts).
    """
    def __new__(cls, value: float) -> Self: ...
    def __add__(self, other: Power) -> Power: ...
    def __sub__(self, other: Power) -> Power: ...
    def __neg__(self) -> Power: ...
    def __mul__(self, other: float) -> Power: ...
    def __rmul__(self, other: float) -> Power: ...
    def __eq__(self, other: object) -> bool: ...
    def __repr__(self) -> str: ...
    def __str__(self) -> str: ...
    def __complex__(self) -> complex: ...
    def __float__(self) -> float: ...
    def __int__(self) -> int: ...
    def to_watts(self) -> float:
        """Returns the value in Watts."""
        ...
    def to_kilowatts(self) -> float:
        """Returns the value in kilowatts."""
        ...
    def to_dbw(self) -> float:
        """Returns the value in dBW."""
        ...

class Temperature:
    """Temperature type for type-safe temperature values.

    Use with unit constants: `290 * lox.K`
    Convert to float with `float(temp)` (returns Kelvin).
    """
    def __new__(cls, value: float) -> Self: ...
    def __add__(self, other: Temperature) -> Temperature: ...
    def __sub__(self, other: Temperature) -> Temperature: ...
    def __neg__(self) -> Temperature: ...
    def __mul__(self, other: float) -> Temperature: ...
    def __rmul__(self, other: float) -> Temperature: ...
    def __eq__(self, other: object) -> bool: ...
    def __repr__(self) -> str: ...
    def __str__(self) -> str: ...
    def __complex__(self) -> complex: ...
    def __float__(self) -> float: ...
    def __int__(self) -> int: ...
    def to_kelvin(self) -> float:
        """Returns the value in Kelvin."""
        ...

class Pressure:
    """Pressure type for type-safe pressure values.

    Use with unit constants: `1013.25 * lox.hPa`
    Convert to float with `float(pressure)` (returns Pa).
    """
    def __new__(cls, value: float) -> Self: ...
    def __add__(self, other: Pressure) -> Pressure: ...
    def __sub__(self, other: Pressure) -> Pressure: ...
    def __neg__(self) -> Pressure: ...
    def __mul__(self, other: float) -> Pressure: ...
    def __rmul__(self, other: float) -> Pressure: ...
    def __eq__(self, other: object) -> bool: ...
    def __repr__(self) -> str: ...
    def __str__(self) -> str: ...
    def __complex__(self) -> complex: ...
    def __float__(self) -> float: ...
    def __int__(self) -> int: ...
    def to_hpa(self) -> float:
        """Returns the value in hectopascals."""
        ...
    def to_pa(self) -> float:
        """Returns the value in pascals."""
        ...

class Velocity:
    """Velocity type for type-safe speed values.

    Use with unit constants: `7.8 * lox.km_per_s` or `100 * lox.m_per_s`
    Convert to float with `float(velocity)` (returns m/s).
    """
    def __new__(cls, value: float) -> Self: ...
    def __add__(self, other: Velocity) -> Velocity: ...
    def __sub__(self, other: Velocity) -> Velocity: ...
    def __neg__(self) -> Velocity: ...
    def __mul__(self, other: float) -> Velocity: ...
    def __rmul__(self, other: float) -> Velocity: ...
    def __eq__(self, other: object) -> bool: ...
    def __repr__(self) -> str: ...
    def __str__(self) -> str: ...
    def __complex__(self) -> complex: ...
    def __float__(self) -> float: ...
    def __int__(self) -> int: ...
    def to_meters_per_second(self) -> float:
        """Returns the value in meters per second."""
        ...
    def to_kilometers_per_second(self) -> float:
        """Returns the value in kilometers per second."""
        ...

# Unit constants
rad: Angle
"""1 radian"""
deg: Angle
"""π/180 radians"""
rad_per_s: AngularRate
"""1 radian per second"""
deg_per_s: AngularRate
"""π/180 radians per second"""
m: Distance
"""1 meter"""
km: Distance
"""1000 meters"""
au: Distance
"""1 astronomical unit"""
Hz: Frequency
"""1 Hz"""
kHz: Frequency
"""1 kHz"""
MHz: Frequency
"""1 MHz"""
GHz: Frequency
"""1 GHz"""
THz: Frequency
"""1 THz"""
W: Power
"""1 Watt"""
kW: Power
"""1000 Watts"""
K: Temperature
"""1 Kelvin"""
Pa: Pressure
"""1 pascal"""
hPa: Pressure
"""100 pascals"""
m_per_s: Velocity
"""1 m/s"""
km_per_s: Velocity
"""1 km/s"""
dB: Decibel
"""1 dB"""
seconds: TimeDelta
"""1 second"""
minutes: TimeDelta
"""60 seconds"""
hours: TimeDelta
"""3600 seconds"""
days: TimeDelta
"""86400 seconds"""

class GroundStation:
    """A named ground station for visibility analysis.

    Wraps a ground location and elevation mask with an identifier.

    Args:
        id: Unique identifier for this ground station.
        location: Ground station location.
        mask: Elevation mask defining minimum elevation constraints.
        comms_payload: Optional communications payload (hardware inventory).

    Examples:
        >>> gs = lox.GroundStation("ESOC", ground_location, elevation_mask)
    """
    def __new__(
        cls,
        id: str,
        location: GroundLocation,
        mask: ElevationMask,
        body_fixed_frame: Frame | None = None,
        network_id: str | None = None,
        comms_payload: CommsPayload | None = None,
    ) -> Self: ...
    def id(self) -> str:
        """Return the asset identifier."""
        ...
    def location(self) -> GroundLocation:
        """Return the ground location."""
        ...
    def mask(self) -> ElevationMask:
        """Return the elevation mask."""
        ...
    def network_id(self) -> str | None:
        """Return the network identifier, if assigned."""
        ...
    def body_fixed_frame(self) -> Frame:
        """Return the body-fixed frame."""
        ...
    def comms_payload(self) -> CommsPayload | None:
        """Return the communications payload, if set."""
        ...

class Spacecraft:
    """A named spacecraft for visibility analysis.

    Wraps an orbit source (propagator or pre-computed trajectory) with an
    identifier.

    Args:
        id: Unique identifier for this spacecraft.
        orbit: Orbit source — an SGP4, Vallado, J2, J4, Numerical
            propagator, or a pre-computed Trajectory.
        max_slew_rate: Optional maximum slew rate.
        comms_payload: Optional communications payload (hardware inventory).

    Examples:
        >>> sc = lox.Spacecraft("ISS", lox.SGP4(tle))
        >>> sc = lox.Spacecraft("SAT", trajectory)
    """
    def __new__(
        cls,
        id: str,
        orbit: SGP4 | Vallado | J2 | J4 | Numerical | Trajectory,
        max_slew_rate: AngularRate | None = None,
        constellation_id: str | None = None,
        optical_payload: "OpticalPayload | None" = None,
        sar_payload: "SarPayload | None" = None,
        comms_payload: CommsPayload | None = None,
    ) -> Self: ...
    def id(self) -> str:
        """Return the asset identifier."""
        ...
    def constellation_id(self) -> str | None:
        """Return the constellation identifier, if assigned."""
        ...
    def max_slew_rate(self) -> AngularRate | None:
        """Return the maximum slew rate, if set."""
        ...
    def optical_payload(self) -> "OpticalPayload | None":
        """Return the optical payload, if set."""
        ...
    def sar_payload(self) -> "SarPayload | None":
        """Return the SAR payload, if set."""
        ...
    def comms_payload(self) -> CommsPayload | None:
        """Return the communications payload, if set."""
        ...

class Scenario:
    """A scenario grouping spacecraft, ground stations, and a time interval.

    Args:
        start: Start time of the scenario.
        end: End time of the scenario.
        spacecraft: List of Spacecraft objects.
        ground_stations: List of GroundStation objects.

    Examples:
        >>> scenario = lox.Scenario(t0, t1, spacecraft=[sc], ground_stations=[gs])
    """
    def __new__(
        cls,
        start: Time,
        end: Time,
        spacecraft: list[Spacecraft] | None = None,
        ground_stations: list[GroundStation] | None = None,
    ) -> Self: ...
    def propagate(self) -> "Ensemble":
        """Propagate all spacecraft, returning an Ensemble.

        Trajectories are transformed to ICRF using the default rotation
        provider.
        """
        ...
    def with_constellation(self, constellation: "Constellation") -> "Scenario":
        """Add a constellation to the scenario.

        Converts each satellite into a Spacecraft using the constellation's
        selected propagator.
        """
        ...
    def start(self) -> Time:
        """Return the start time."""
        ...
    def end(self) -> Time:
        """Return the end time."""
        ...

class Ensemble:
    """A collection of propagated trajectories keyed by spacecraft id.

    Examples:
        >>> ensemble = scenario.propagate()
        >>> traj = ensemble.get("ISS")
    """
    def get(self, id: str) -> Trajectory | None:
        """Return the trajectory for a given spacecraft id."""
        ...
    def __len__(self) -> int: ...
    def __repr__(self) -> str: ...

class ConstellationSatellite:
    """A single satellite in a constellation."""
    @property
    def plane(self) -> int:
        """Return the orbital plane index (0-based)."""
        ...
    @property
    def index_in_plane(self) -> int:
        """Return the index within the plane (0-based)."""
        ...
    def __repr__(self) -> str: ...

class Constellation:
    """A named collection of satellites produced by a constellation design algorithm.

    Use the classmethods to create constellations of different types.
    """
    @classmethod
    def walker_delta(
        cls,
        name: str,
        time: Time,
        origin: Origin,
        *,
        nsats: int,
        nplanes: int,
        semi_major_axis: Distance,
        inclination: Angle,
        eccentricity: float = 0.0,
        phasing: int = 0,
        argument_of_periapsis: Angle | None = None,
        longitude_of_ascending_node: Angle | None = None,
        propagator: str = "vallado",
    ) -> "Constellation":
        """Create a Walker Delta constellation (RAAN spread = 360 deg)."""
        ...
    @classmethod
    def walker_star(
        cls,
        name: str,
        time: Time,
        origin: Origin,
        *,
        nsats: int,
        nplanes: int,
        semi_major_axis: Distance,
        inclination: Angle,
        eccentricity: float = 0.0,
        phasing: int = 0,
        argument_of_periapsis: Angle | None = None,
        longitude_of_ascending_node: Angle | None = None,
        propagator: str = "vallado",
    ) -> "Constellation":
        """Create a Walker Star constellation (RAAN spread = 180 deg)."""
        ...
    @classmethod
    def street_of_coverage(
        cls,
        name: str,
        time: Time,
        origin: Origin,
        *,
        nsats: int,
        nplanes: int,
        semi_major_axis: Distance,
        inclination: Angle,
        eccentricity: float = 0.0,
        coverage_fold: int = 1,
        argument_of_periapsis: Angle | None = None,
        longitude_of_ascending_node: Angle | None = None,
        propagator: str = "vallado",
    ) -> "Constellation":
        """Create a Street-of-Coverage constellation."""
        ...
    @classmethod
    def flower(
        cls,
        name: str,
        time: Time,
        origin: Origin,
        *,
        n_petals: int,
        n_days: int,
        nsats: int,
        phasing_numerator: int,
        phasing_denominator: int,
        inclination: Angle,
        perigee_altitude: Distance | None = None,
        semi_major_axis: Distance | None = None,
        eccentricity: float | None = None,
        argument_of_periapsis: Angle | None = None,
        longitude_of_ascending_node: Angle | None = None,
        propagator: str = "vallado",
    ) -> "Constellation":
        """Create a Flower constellation (repeating ground tracks)."""
        ...
    @property
    def name(self) -> str:
        """Return the constellation name."""
        ...
    @property
    def satellites(self) -> list[ConstellationSatellite]:
        """Return the list of satellites."""
        ...
    def __len__(self) -> int: ...
    def __repr__(self) -> str: ...

class PowerBudgetAnalysis:
    """Power budget analysis for spacecraft in a scenario.

    Computes eclipse intervals, sun beta angle, and solar flux for each
    spacecraft.  The shadow model is cylindrical (umbra only) — penumbra
    is **not** modelled.

    Args:
        scenario: Scenario containing spacecraft and time interval.
        ensemble: Optional pre-computed Ensemble.
        step: Optional time step for sampling / event detection (default: 60s).
        spacecraft_ids: Optional list of spacecraft ids to restrict the analysis.

    Examples:
        >>> analysis = lox.PowerBudgetAnalysis(scenario)
        >>> results = analysis.compute()          # analytical Sun
        >>> results = analysis.compute(ephemeris)  # SPK Sun
    """
    def __new__(
        cls,
        scenario: Scenario,
        ensemble: Ensemble | None = None,
        step: TimeDelta | None = None,
        spacecraft_ids: list[str] | None = None,
        constellation_id: str | None = None,
    ) -> Self: ...
    def compute(self, ephemeris: SPK | None = None) -> "PowerBudgetResults":
        """Compute the power budget analysis.

        Args:
            ephemeris: Optional SPK ephemeris for Sun position. When omitted,
                an analytical model is used (valid for Earth-centred scenarios).

        Returns:
            PowerBudgetResults with eclipse intervals, beta angles, and
            solar flux for each spacecraft.
        """
        ...

class PowerBudgetResults:
    """Results of a power budget analysis.

    Provides access to eclipse intervals, eclipse/sunlit fractions,
    beta-angle time series, and solar-flux time series for each spacecraft.
    """
    def eclipse_intervals(self, id: str) -> list[Interval]:
        """Return eclipse intervals for a specific spacecraft.

        Args:
            id: Spacecraft identifier.

        Returns:
            List of Interval objects, or empty list if id not found.
        """
        ...
    def eclipse_fraction(self, id: str) -> float | None:
        """Return eclipse fraction (0 = fully sunlit, 1 = always eclipsed).

        Args:
            id: Spacecraft identifier.

        Returns:
            Eclipse fraction, or None if id not found.
        """
        ...
    def sunlit_fraction(self, id: str) -> float | None:
        """Return sunlit fraction (1 - eclipse_fraction).

        Args:
            id: Spacecraft identifier.

        Returns:
            Sunlit fraction, or None if id not found.
        """
        ...
    def beta_angles(self, id: str) -> TimeSeries | None:
        """Return beta-angle time series (radians).

        Args:
            id: Spacecraft identifier.

        Returns:
            TimeSeries of beta angles in radians, or None if id not found.
        """
        ...
    def solar_flux(self, id: str) -> TimeSeries | None:
        """Return solar-flux time series (W/m²).

        Args:
            id: Spacecraft identifier.

        Returns:
            TimeSeries of solar flux in W/m², or None if id not found.
        """
        ...

class VisibilityAnalysis:
    """Computes ground-station-to-spacecraft and inter-satellite visibility.

    Ground-to-space pairs are always computed when ground assets are present.
    Inter-satellite pairs are additionally computed when ``inter_satellite``
    is set to True.

    Args:
        scenario: Scenario containing spacecraft, ground stations, and
            time interval.
        ensemble: Optional pre-computed Ensemble. If not provided, the
            scenario is propagated automatically.
        occulting_bodies: Optional list of additional occulting bodies for
            line-of-sight checking. For inter-satellite visibility, the
            scenario's central body is always checked automatically.
        step: Optional time step for event detection (default: 60s).
        min_pass_duration: Optional minimum pass duration. Passes
            shorter than this value may be missed. Enables two-level stepping
            for faster detection.
        inter_satellite: If True, also compute inter-satellite visibility
            for all unique spacecraft pairs (default: False).
        ground_space_filter: Optional callable
            ``(GroundStation, Spacecraft) -> bool`` that receives a ground
            station and a spacecraft and returns whether the pair should be
            evaluated. Called once per candidate pair before the parallel
            phase.
        inter_satellite_filter: Optional callable
            ``(Spacecraft, Spacecraft) -> bool`` that receives two spacecraft
            and returns whether the pair should be evaluated. Called once per
            candidate pair before the parallel phase. When provided,
            inter-satellite visibility is automatically enabled.
        min_range: Optional minimum range constraint for inter-satellite pairs.
        max_range: Optional maximum range constraint for inter-satellite pairs.

    Examples:
        >>> scenario = lox.Scenario(t0, t1, spacecraft=[sc], ground_stations=[gs])
        >>> analysis = lox.VisibilityAnalysis(scenario, step=lox.TimeDelta(60))
        >>> results = analysis.compute(spk)
    """
    def __new__(
        cls,
        scenario: Scenario,
        ensemble: Ensemble | None = None,
        occulting_bodies: list[str | int | Origin] | None = None,
        step: TimeDelta | None = None,
        min_pass_duration: TimeDelta | None = None,
        inter_satellite: bool = False,
        ground_space_filter: Callable[[GroundStation, Spacecraft], bool] | None = None,
        inter_satellite_filter: Callable[[Spacecraft, Spacecraft], bool] | None = None,
        min_range: Distance | None = None,
        max_range: Distance | None = None,
    ) -> Self: ...
    def compute(self, ephemeris: SPK) -> VisibilityResults:
        """Compute visibility intervals for all pairs.

        If no ensemble was provided at construction, the scenario is
        propagated automatically (trajectories transformed to ICRF).

        Args:
            ephemeris: SPK ephemeris data.

        Returns:
            VisibilityResults containing intervals for all pairs.
        """
        ...

class VisibilityResults:
    """Results of a visibility analysis.

    Provides access to visibility intervals and passes. Intervals (time
    windows) are computed eagerly; observables-rich Pass objects are
    computed on demand.
    """
    def intervals(self, id1: str, id2: str) -> list[Interval]:
        """Return visibility intervals for a specific pair.

        Args:
            id1: First asset identifier (ground or space).
            id2: Second asset identifier (space).

        Returns:
            List of Interval objects, or empty list if pair not found.
        """
        ...
    def all_intervals(self) -> dict[tuple[str, str], list[Interval]]:
        """Return all intervals for all pairs.

        Returns:
            Dictionary mapping (id1, id2) to list of Interval objects.
        """
        ...
    def ground_space_intervals(self) -> dict[tuple[str, str], list[Interval]]:
        """Return intervals for ground-to-space pairs only.

        Returns:
            Dictionary mapping (ground_id, space_id) to list of Interval objects.
        """
        ...
    def inter_satellite_intervals(self) -> dict[tuple[str, str], list[Interval]]:
        """Return intervals for inter-satellite pairs only.

        Returns:
            Dictionary mapping (sc1_id, sc2_id) to list of Interval objects.
        """
        ...
    def passes(self, ground_id: str, space_id: str) -> list[Pass]:
        """Compute passes with observables for a specific ground-to-space pair.

        This is more expensive than ``intervals()`` as it computes azimuth,
        elevation, range, and range rate for each time step.

        Raises:
            ValueError: If the pair is an inter-satellite pair.

        Args:
            ground_id: Ground asset identifier.
            space_id: Space asset identifier.

        Returns:
            List of Pass objects, or empty list if pair not found.
        """
        ...
    def all_passes(self) -> dict[tuple[str, str], list[Pass]]:
        """Compute passes for all ground-to-space pairs.

        Inter-satellite pairs are skipped.

        Returns:
            Dictionary mapping (ground_id, space_id) to list of Pass objects.
        """
        ...
    def pair_ids(self) -> list[tuple[str, str]]:
        """Return all pair identifiers."""
        ...
    def ground_space_pair_ids(self) -> list[tuple[str, str]]:
        """Return pair identifiers for ground-to-space pairs only."""
        ...
    def inter_satellite_pair_ids(self) -> list[tuple[str, str]]:
        """Return pair identifiers for inter-satellite pairs only."""
        ...
    def num_pairs(self) -> int:
        """Return the total number of pairs."""
        ...
    def total_intervals(self) -> int:
        """Return the total number of visibility intervals across all pairs."""
        ...

class ElevationMask:
    """Defines elevation constraints for visibility analysis.

    An elevation mask specifies the minimum elevation angle required for
    visibility at different azimuth angles. Can be either fixed (constant)
    or variable (azimuth-dependent).

    Args:
        azimuth: Array of azimuth angles in radians (for variable mask).
        elevation: Array of minimum elevations in radians (for variable mask).
        min_elevation: Fixed minimum elevation as Angle.

    Examples:
        >>> # Fixed elevation mask (5 degrees)
        >>> mask = lox.ElevationMask.fixed(5.0 * lox.deg)

        >>> # Variable mask based on terrain
        >>> mask = lox.ElevationMask.variable(azimuth, elevation)
    """
    def __new__(
        cls,
        azimuth: np.ndarray | None = None,
        elevation: np.ndarray | None = None,
        min_elevation: Angle | None = None,
    ) -> Self: ...
    @classmethod
    def variable(cls, azimuth: np.ndarray, elevation: np.ndarray) -> Self:
        """Create a variable elevation mask from azimuth-dependent data."""
        ...
    @classmethod
    def fixed(cls, min_elevation: Angle) -> Self:
        """Create a fixed elevation mask with constant minimum elevation."""
        ...
    def azimuth(self) -> list[float] | None:
        """Return the azimuth array (for variable masks only)."""
        ...
    def elevation(self) -> list[float] | None:
        """Return the elevation array (for variable masks only)."""
        ...
    def fixed_elevation(self) -> Angle | None:
        """Return the fixed elevation value (for fixed masks only)."""
        ...
    def min_elevation(self, azimuth: Angle) -> Angle:
        """Return the minimum elevation at the given azimuth."""
        ...

def find_events(
    func: Callable[[Time], float], start: Time, end: Time, step: TimeDelta
) -> list[Event]:
    """Find events where a function crosses zero.

    Args:
        func: Function that takes a Time and returns a float.
        start: Start time of the analysis period.
        end: End time of the analysis period.
        step: Step size for sampling the function.

    Returns:
        List of Event objects at the detected zero-crossings.
    """
    ...

def find_windows(
    func: Callable[[Time], float], start: Time, end: Time, step: TimeDelta
) -> list[Interval]:
    """Find time windows where a function is positive.

    Args:
        func: Function that takes a Time and returns a float.
        start: Start time of the analysis period.
        end: End time of the analysis period.
        step: Step size for sampling the function.

    Returns:
        List of Interval objects for intervals where the function is positive.
    """
    ...

def intersect_intervals(a: list[Interval], b: list[Interval]) -> list[Interval]:
    """Intersect two sorted lists of intervals.

    Args:
        a: First list of intervals.
        b: Second list of intervals.

    Returns:
        List of intervals representing the intersection.
    """
    ...

def union_intervals(a: list[Interval], b: list[Interval]) -> list[Interval]:
    """Compute the union of two sorted lists of intervals.

    Args:
        a: First list of intervals.
        b: Second list of intervals.

    Returns:
        List of merged intervals representing the union.
    """
    ...

def complement_intervals(intervals: list[Interval], bound: Interval) -> list[Interval]:
    """Compute the complement of intervals within a bounding interval.

    Args:
        intervals: List of intervals to complement.
        bound: Bounding interval.

    Returns:
        List of gap intervals within the bound.
    """
    ...

class Origin:
    """Represents a celestial body (planet, moon, barycenter, etc.).

    Origin objects represent celestial bodies using NAIF/SPICE identifiers.
    They provide access to physical properties such as gravitational parameters,
    radii, and rotational elements.

    Args:
        origin: Body name (e.g., "Earth", "Moon") or NAIF ID (e.g., 399 for Earth).

    Raises:
        ValueError: If the origin name or ID is not recognized.
        TypeError: If the argument is neither a string nor an integer.

    Examples:
        >>> earth = lox.Origin("Earth")
        >>> moon = lox.Origin("Moon")
        >>> mars = lox.Origin(499)  # NAIF ID
    """
    def __new__(cls, origin: str | int) -> Self: ...
    def __repr__(self) -> str: ...
    def __str__(self) -> str: ...
    def id(self) -> int:
        """Return the NAIF ID of this body."""
        ...
    def name(self) -> str:
        """Return the name of this body."""
        ...
    def gravitational_parameter(self) -> GravitationalParameter:
        """Return the gravitational parameter (GM)."""
        ...
    def mean_radius(self) -> Distance:
        """Return the mean radius."""
        ...
    def radii(self) -> tuple[Distance, Distance, Distance]:
        """Return the triaxial radii (x, y, z)."""
        ...
    def equatorial_radius(self) -> Distance:
        """Return the equatorial radius."""
        ...
    def polar_radius(self) -> Distance:
        """Return the polar radius."""
        ...
    def rotational_elements(self, et: float) -> tuple[Angle, Angle, Angle]:
        """Return rotational elements (right ascension, declination, rotation angle)."""
        ...
    def rotational_element_rates(
        self, et: float
    ) -> tuple[AngularRate, AngularRate, AngularRate]:
        """Return rotational element rates."""
        ...
    def right_ascension(self, et: float) -> Angle:
        """Return the right ascension of the pole."""
        ...
    def right_ascension_rate(self, et: float) -> AngularRate:
        """Return the rate of change of right ascension."""
        ...
    def declination(self, et: float) -> Angle:
        """Return the declination of the pole."""
        ...
    def declination_rate(self, et: float) -> AngularRate:
        """Return the rate of change of declination."""
        ...
    def rotation_angle(self, et: float) -> Angle:
        """Return the rotation angle (prime meridian)."""
        ...
    def rotation_rate(self, et: float) -> AngularRate:
        """Return the rotation rate."""
        ...

class Frame:
    """Represents a reference frame for positioning and transformations.

    Supported frames:
    - ICRF: International Celestial Reference Frame (inertial)
    - GCRF: Geocentric Celestial Reference Frame (inertial, Earth-centered)
    - CIRF: Celestial Intermediate Reference Frame
    - TIRF: Terrestrial Intermediate Reference Frame
    - ITRF: International Terrestrial Reference Frame (Earth-fixed)
    - Body-fixed frames: IAU_EARTH, IAU_MOON, IAU_MARS, etc.

    Args:
        abbreviation: Frame abbreviation (e.g., "ICRF", "ITRF", "IAU_MOON").

    Raises:
        ValueError: If the frame abbreviation is not recognized.

    Examples:
        >>> icrf = lox.Frame("ICRF")
        >>> itrf = lox.Frame("ITRF")
    """
    def __new__(cls, abbreviation: str) -> Self: ...
    def name(self) -> str:
        """Return the full name of this reference frame."""
        ...
    def abbreviation(self) -> str:
        """Return the abbreviation of this reference frame."""
        ...

class SPK:
    """SPICE SPK (Spacecraft and Planet Kernel) ephemeris data.

    SPK files contain position and velocity data for celestial bodies and
    spacecraft. They are used for orbit propagation, frame transformations,
    and visibility analysis.

    Args:
        path: Path to the SPK file (.bsp).

    Raises:
        ValueError: If the file cannot be parsed or is invalid.
        OSError: If the file cannot be read.

    Examples:
        >>> spk = lox.SPK("/path/to/de440.bsp")
    """
    def __new__(cls, path: os.PathLike) -> Self: ...

class Cartesian:
    """Represents an orbital state (position and velocity) at a specific time.

    Args:
        time: The epoch of this state.
        position: Position vector as array-like [x, y, z] in meters.
        velocity: Velocity vector as array-like [vx, vy, vz] in m/s.
        x, y, z: Individual position components as Distance (keyword-only, alternative to position).
        vx, vy, vz: Individual velocity components as Velocity (keyword-only, alternative to velocity).
        origin: Central body (default: Earth).
        frame: Reference frame (default: ICRF).

    Examples:
        >>> t = lox.Time("TAI", 2024, 1, 1)
        >>> # Array pattern (meters, m/s)
        >>> state = lox.Cartesian(
        ...     t,
        ...     position=[6678e3, 0.0, 0.0],
        ...     velocity=[0.0, 7730.0, 0.0],
        ... )
        >>> # Elementwise pattern (unitful)
        >>> state = lox.Cartesian(
        ...     t,
        ...     x=6678.0 * lox.km, y=0.0 * lox.km, z=0.0 * lox.km,
        ...     vx=0.0 * lox.km_per_s, vy=7.73 * lox.km_per_s, vz=0.0 * lox.km_per_s,
        ... )
    """
    def __new__(
        cls,
        time: Time,
        position: ArrayLike | None = None,
        velocity: ArrayLike | None = None,
        *,
        x: Distance | None = None,
        y: Distance | None = None,
        z: Distance | None = None,
        vx: Velocity | None = None,
        vy: Velocity | None = None,
        vz: Velocity | None = None,
        origin: str | int | Origin | None = None,
        frame: str | Frame | None = None,
    ) -> Self: ...
    def time(self) -> Time:
        """Return the epoch of this state."""
        ...
    def origin(self) -> Origin:
        """Return the central body (origin) of this state."""
        ...
    def reference_frame(self) -> Frame:
        """Return the reference frame of this state."""
        ...
    def position(self) -> np.ndarray:
        """Return the position vector as a numpy array in meters, shape (3,)."""
        ...
    def velocity(self) -> np.ndarray:
        """Return the velocity vector as a numpy array in m/s, shape (3,)."""
        ...
    @property
    def x(self) -> Distance:
        """Return the x component of the position."""
        ...
    @property
    def y(self) -> Distance:
        """Return the y component of the position."""
        ...
    @property
    def z(self) -> Distance:
        """Return the z component of the position."""
        ...
    @property
    def vx(self) -> Velocity:
        """Return the x component of the velocity."""
        ...
    @property
    def vy(self) -> Velocity:
        """Return the y component of the velocity."""
        ...
    @property
    def vz(self) -> Velocity:
        """Return the z component of the velocity."""
        ...
    def to_frame(self, frame: str | Frame, provider: EOPProvider | None = None) -> Self:
        """Transform this state to a different reference frame."""
        ...
    def to_origin(self, target: str | int | Origin, ephemeris: SPK) -> Self:
        """Transform this state to a different central body."""
        ...
    def to_keplerian(self) -> Keplerian:
        """Convert this Cartesian state to Keplerian orbital elements."""
        ...
    def to_modified_equinoctial(self) -> ModifiedEquinoctial:
        """Convert this Cartesian state to modified equinoctial elements."""
        ...
    def rotation_lvlh(self) -> np.ndarray:
        """Compute the rotation matrix from inertial to LVLH frame."""
        ...
    def to_ground_location(self) -> GroundLocation:
        """Convert this state to a ground location."""
        ...

# Keep old name as alias

class Keplerian:
    """Represents an orbit using Keplerian (classical) orbital elements.

    The orbital shape can be specified in three ways:

    - ``semi_major_axis`` + ``eccentricity``
    - ``periapsis_radius`` + ``apoapsis_radius`` (keyword-only)
    - ``periapsis_altitude`` + ``apoapsis_altitude`` (keyword-only)

    Args:
        time: Epoch of the elements.
        semi_major_axis: Semi-major axis as Distance.
        eccentricity: Orbital eccentricity (0 = circular, <1 = elliptical).
        inclination: Inclination as Angle (default 0).
        longitude_of_ascending_node: RAAN as Angle (default 0).
        argument_of_periapsis: Argument of periapsis as Angle (default 0).
        true_anomaly: True anomaly as Angle (default 0).
        origin: Central body (default: Earth).
        periapsis_radius: Periapsis radius as Distance (keyword-only).
        apoapsis_radius: Apoapsis radius as Distance (keyword-only).
        periapsis_altitude: Periapsis altitude as Distance (keyword-only).
        apoapsis_altitude: Apoapsis altitude as Distance (keyword-only).
        mean_anomaly: Mean anomaly as Angle (keyword-only, mutually exclusive with true_anomaly).

    Examples:
        >>> t = lox.Time("TAI", 2024, 1, 1)
        >>> orbit = lox.Keplerian(
        ...     t,
        ...     semi_major_axis=6678.0 * lox.km,
        ...     eccentricity=0.001,
        ...     inclination=51.6 * lox.deg,
        ... )

        From radii:

        >>> orbit = lox.Keplerian(
        ...     t,
        ...     periapsis_radius=7000.0 * lox.km,
        ...     apoapsis_radius=7400.0 * lox.km,
        ... )

        From altitudes:

        >>> orbit = lox.Keplerian(
        ...     t,
        ...     periapsis_altitude=600.0 * lox.km,
        ...     apoapsis_altitude=1000.0 * lox.km,
        ... )
    """
    def __new__(
        cls,
        time: Time,
        semi_major_axis: Distance | None = None,
        eccentricity: float | None = None,
        inclination: Angle | None = None,
        longitude_of_ascending_node: Angle | None = None,
        argument_of_periapsis: Angle | None = None,
        true_anomaly: Angle | None = None,
        origin: str | int | Origin | None = None,
        *,
        periapsis_radius: Distance | None = None,
        apoapsis_radius: Distance | None = None,
        periapsis_altitude: Distance | None = None,
        apoapsis_altitude: Distance | None = None,
        mean_anomaly: Angle | None = None,
    ) -> Self: ...
    @classmethod
    def circular(
        cls,
        time: Time,
        *,
        semi_major_axis: Distance | None = None,
        altitude: Distance | None = None,
        inclination: Angle | None = None,
        longitude_of_ascending_node: Angle | None = None,
        true_anomaly: Angle | None = None,
        origin: str | int | Origin | None = None,
    ) -> Self:
        """Construct a circular orbit.

        Exactly one of ``semi_major_axis`` or ``altitude`` must be provided.
        Eccentricity is always 0 and argument of periapsis is always 0.

        Args:
            time: Epoch of the orbit.
            semi_major_axis: Semi-major axis (mutually exclusive with altitude).
            altitude: Orbital altitude (mutually exclusive with semi_major_axis).
            inclination: Inclination (default 0).
            longitude_of_ascending_node: RAAN (default 0).
            true_anomaly: True anomaly (default 0).
            origin: Central body (default: Earth).

        Examples:
            >>> t = lox.Time("TAI", 2024, 1, 1)
            >>> orbit = lox.Keplerian.circular(
            ...     t,
            ...     altitude=800 * lox.km,
            ...     inclination=51.6 * lox.deg,
            ... )
        """
        ...
    @classmethod
    def sso(
        cls,
        time: Time,
        *,
        altitude: Distance | None = None,
        semi_major_axis: Distance | None = None,
        inclination: Angle | None = None,
        eccentricity: float = 0.0,
        ltan: tuple[int, int] | None = None,
        ltdn: tuple[int, int] | None = None,
        argument_of_periapsis: Angle | None = None,
        true_anomaly: Angle | None = None,
        provider: EOPProvider | None = None,
    ) -> Self:
        """Construct a Sun-synchronous orbit.

        Exactly one of ``altitude``, ``semi_major_axis``, or ``inclination``
        must be provided.  The remaining orbital elements are derived from the
        SSO constraint.

        Args:
            time: Epoch of the orbit.
            altitude: Orbital altitude (mutually exclusive with semi_major_axis/inclination).
            semi_major_axis: Semi-major axis (mutually exclusive with altitude/inclination).
            inclination: Inclination (mutually exclusive with altitude/semi_major_axis).
            eccentricity: Eccentricity (default 0.0).
            ltan: Local time of ascending node as ``(hours, minutes)`` tuple.
            ltdn: Local time of descending node as ``(hours, minutes)`` tuple.
            argument_of_periapsis: Argument of periapsis (default 0.0).
            true_anomaly: True anomaly (default 0.0).
            provider: EOP provider for time scale conversions.

        Examples:
            >>> eop = lox.EOPProvider("finals.csv")
            >>> t = lox.UTC(2020, 2, 18).to_scale("TDB")
            >>> orbit = lox.Keplerian.sso(
            ...     t,
            ...     altitude=800 * lox.km,
            ...     ltan=(13, 30),
            ...     provider=eop,
            ... )
        """
        ...
    def time(self) -> Time:
        """Return the epoch of these elements."""
        ...
    def origin(self) -> Origin:
        """Return the central body (origin) of this orbit."""
        ...
    def semi_major_axis(self) -> Distance:
        """Return the semi-major axis."""
        ...
    def eccentricity(self) -> float:
        """Return the orbital eccentricity."""
        ...
    def inclination(self) -> Angle:
        """Return the inclination."""
        ...
    def longitude_of_ascending_node(self) -> Angle:
        """Return the longitude of the ascending node (RAAN)."""
        ...
    def argument_of_periapsis(self) -> Angle:
        """Return the argument of periapsis."""
        ...
    def true_anomaly(self) -> Angle:
        """Return the true anomaly."""
        ...
    def to_cartesian(self) -> Cartesian:
        """Convert these Keplerian elements to a Cartesian state."""
        ...
    def to_modified_equinoctial(self) -> ModifiedEquinoctial:
        """Convert these Keplerian elements to modified equinoctial elements."""
        ...
    def orbital_period(self) -> TimeDelta:
        """Return the orbital period."""
        ...

class ModifiedEquinoctial:
    """Represents an orbit using Modified Equinoctial Elements (MEE).

    Modified Equinoctial Elements are non-singular for circular (e = 0) and equatorial (i = 0)
    orbits. They also fully support parabolic (e = 1) orbits.

    Args:
        time: Epoch of the elements.
        p: Semi-latus rectum (semi-parameter) as Distance.
        f: Eccentricity vector component 1.
        g: Eccentricity vector component 2.
        h: Node vector component 1.
        k: Node vector component 2.
        l: True longitude as Angle.
        origin: Central body (default: Earth).
        frame: Reference frame (default: ICRF).

    Examples:
        >>> t = lox.Time("TAI", 2024, 1, 1)
        >>> mee = lox.ModifiedEquinoctial(
        ...     t,
        ...     p=7000.0 * lox.km,
        ...     f=0.001,
        ...     g=0.001,
        ...     h=0.0,
        ...     k=0.0,
        ...     l=0.0 * lox.deg,
        ... )
    """
    def __new__(
        cls,
        time: Time,
        p: Distance,
        f: float,
        g: float,
        h: float,
        k: float,
        l: Angle,
        origin: str | int | Origin | None = None,
        frame: str | Frame | None = None,
    ) -> Self: ...
    def time(self) -> Time:
        """Return the epoch of this orbit."""
        ...
    def origin(self) -> Origin:
        """Return the central body (origin) of this orbit."""
        ...
    def frame(self) -> Frame:
        """Return the reference frame."""
        ...
    def p(self) -> Distance:
        """Return the semi-latus rectum (semi-parameter) `p`."""
        ...
    def f(self) -> float:
        """Return `f` = e·cos(ω + Ω)."""
        ...
    def g(self) -> float:
        """Return `g` = e·sin(ω + Ω)."""
        ...
    def h(self) -> float:
        """Return `h` = tan(i/2)·cos(Ω)."""
        ...
    def k(self) -> float:
        """Return `k` = tan(i/2)·sin(Ω)."""
        ...
    def l(self) -> Angle:
        """Return the true longitude `l` = Ω + ω + ν."""
        ...
    def eccentricity(self) -> float:
        """Return the orbital eccentricity."""
        ...
    def inclination(self) -> Angle:
        """Return the orbital inclination."""
        ...
    def to_cartesian(self) -> Cartesian:
        """Convert these modified equinoctial elements to a Cartesian state."""
        ...
    def to_keplerian(self) -> Keplerian:
        """Convert these modified equinoctial elements to Keplerian orbital elements."""
        ...

class Trajectory:
    """A time-series of orbital states with interpolation support.

    Args:
        states: List of Cartesian objects in chronological order.

    Examples:
        >>> trajectory = propagator.propagate(times)
        >>> state = trajectory.interpolate(t)
        >>> arr = trajectory.to_numpy()
    """
    def __new__(cls, states: list[Cartesian]) -> Self: ...
    @classmethod
    def from_numpy(
        cls,
        start_time: Time,
        states: np.ndarray,
        origin: str | int | Origin | None = None,
        frame: str | Frame | None = None,
    ) -> Self:
        """Create a Trajectory from a numpy array.

        Args:
            start_time: Reference epoch for the trajectory.
            states: 2D array with columns [t(s), x(m), y(m), z(m), vx(m/s), vy(m/s), vz(m/s)].
            origin: Central body (default: Earth).
            frame: Reference frame (default: ICRF).
        """
        ...
    def origin(self) -> Origin:
        """Return the central body (origin) of this trajectory."""
        ...
    def reference_frame(self) -> Frame:
        """Return the reference frame of this trajectory."""
        ...
    def to_numpy(self) -> np.ndarray:
        """Export trajectory to a numpy array with columns [t(s), x(m), y(m), z(m), vx(m/s), vy(m/s), vz(m/s)]."""
        ...
    def states(self) -> list[Cartesian]:
        """Return the list of states in this trajectory."""
        ...
    def find_events(
        self, func: Callable[[Cartesian], float], step: TimeDelta
    ) -> list[Event]:
        """Find events where a function crosses zero."""
        ...
    def find_windows(
        self, func: Callable[[Cartesian], float], step: TimeDelta
    ) -> list[Interval]:
        """Find time windows where a function is positive."""
        ...
    def interpolate(self, time: Time | TimeDelta) -> Cartesian:
        """Interpolate the trajectory at a specific time."""
        ...
    def to_frame(self, frame: str | Frame, provider: EOPProvider | None = None) -> Self:
        """Transform all states to a different reference frame."""
        ...
    def to_origin(self, target: str | int | Origin, ephemeris: SPK) -> Self:
        """Transform all states to a different central body."""
        ...

class Event:
    """Represents a detected event (zero-crossing of a function).

    Events are detected when a monitored function crosses zero.
    The crossing direction indicates "up" (negative to positive)
    or "down" (positive to negative).
    """
    def __repr__(self) -> str: ...
    def __str__(self) -> str: ...
    def time(self) -> Time:
        """Return the time of this event."""
        ...
    def crossing(self) -> str:
        """Return the crossing direction ("up" or "down")."""
        ...

class Interval:
    """Represents a time interval between two times.

    Intervals represent periods when certain conditions are met,
    such as visibility intervals between a ground station and spacecraft.
    """
    def __new__(cls, start: Time, end: Time) -> Self: ...
    def __repr__(self) -> str: ...
    def start(self) -> Time:
        """Return the start time of this interval."""
        ...
    def end(self) -> Time:
        """Return the end time of this interval."""
        ...
    def duration(self) -> TimeDelta:
        """Return the duration of this interval."""
        ...
    def is_empty(self) -> bool:
        """Return whether this interval is empty (start >= end)."""
        ...
    def contains_time(self, time: Time) -> bool:
        """Return whether this interval contains the given time."""
        ...
    def contains(self, other: Interval) -> bool:
        """Return whether this interval fully contains another interval."""
        ...
    def intersect(self, other: Interval) -> Interval:
        """Return the intersection of this interval with another."""
        ...
    def overlaps(self, other: Interval) -> bool:
        """Return whether this interval overlaps with another."""
        ...
    def step_by(self, step: TimeDelta) -> list[Time]:
        """Return a list of times spaced by the given step within this interval.

        Raises:
            ValueError: If step is zero.
        """
        ...
    def linspace(self, n: int) -> list[Time]:
        """Return a list of n evenly-spaced times within this interval.

        Raises:
            ValueError: If n < 2.
        """
        ...

class Vallado:
    """Semi-analytical Keplerian orbit propagator using Vallado's method.

    Args:
        initial_state: Initial orbital state (must be in an inertial frame).
        max_iter: Maximum iterations for Kepler's equation solver.

    Examples:
        >>> state = lox.Cartesian(t,
        ...     position=(6678.0 * km, 0.0 * km, 0.0 * km),
        ...     velocity=(0.0 * km_per_s, 7.73 * km_per_s, 0.0 * km_per_s))
        >>> prop = lox.Vallado(state)
        >>> trajectory = prop.propagate([t1, t2, t3])
    """
    def __new__(cls, initial_state: Cartesian, max_iter: int | None = None) -> Self: ...
    @overload
    def propagate(self, steps: Time) -> Cartesian: ...
    @overload
    def propagate(self, steps: list[Time]) -> Trajectory: ...
    def propagate(
        self,
        steps: Time | list[Time],
        end: Time | None = None,
        frame: str | Frame | None = None,
        provider: EOPProvider | None = None,
    ) -> Cartesian | Trajectory:
        """Propagate the orbit to one or more times."""
        ...

class Numerical:
    """Numerical orbit propagator using Dormand-Prince 8(5,3) integration.

    This propagator accounts for the J2 zonal harmonic perturbation, which
    models the oblateness of the central body. It uses an adaptive Runge-Kutta
    integrator (DOP853).

    Args:
        initial_state: Initial orbital state.
        rtol: Relative tolerance (default: 1e-8).
        atol: Absolute tolerance (default: 1e-6).
        h_max: Maximum step size in seconds (default: auto from orbital timescale).
        h_min: Minimum step size in seconds (default: 1e-6).
        max_steps: Maximum number of integration steps (default: 100000).
    """
    def __new__(
        cls,
        initial_state: Cartesian,
        rtol: float | None = None,
        atol: float | None = None,
        h_max: float | None = None,
        h_min: float | None = None,
        max_steps: int | None = None,
    ) -> Self: ...
    @overload
    def propagate(self, steps: Time) -> Cartesian: ...
    @overload
    def propagate(self, steps: list[Time]) -> Trajectory: ...
    def propagate(
        self,
        steps: Time | list[Time],
        end: Time | None = None,
        frame: str | Frame | None = None,
        provider: EOPProvider | None = None,
    ) -> Cartesian | Trajectory:
        """Propagate the orbit to one or more times."""
        ...

class J2:
    """Analytical J2 orbit propagator (Kozai secular ± Kwok short-period).

    Propagates mean Keplerian elements with first-order J2 secular rates.
    Optionally applies Kwok short-period corrections for osculating output.
    Works for all orbit types including circular (e = 0).

    Args:
        initial_state: Initial orbital state (mean elements).
        osculating: Enable Kwok short-period corrections (default: False).
        step: Fixed time step in seconds for interval propagation (default: 60).
    """
    def __new__(
        cls,
        initial_state: Cartesian,
        osculating: bool = False,
        step: float | None = None,
    ) -> Self: ...
    @overload
    def propagate(self, steps: Time) -> Cartesian: ...
    @overload
    def propagate(self, steps: list[Time]) -> Trajectory: ...
    def propagate(
        self,
        steps: Time | list[Time],
        end: Time | None = None,
        frame: str | Frame | None = None,
        provider: EOPProvider | None = None,
    ) -> Cartesian | Trajectory:
        """Propagate the orbit to one or more times."""
        ...

class J4:
    """Analytical J4 orbit propagator (Kozai secular with J2²+J4 ± Kwok short-period).

    Like ``J2`` but uses higher-order secular rates including J2² and J4
    zonal harmonic terms. Short-period corrections (when enabled) are
    J2-only. Works for all orbit types including circular (e = 0).

    Args:
        initial_state: Initial orbital state (mean elements).
        osculating: Enable Kwok short-period corrections (default: False).
        step: Fixed time step in seconds for interval propagation (default: 60).
    """
    def __new__(
        cls,
        initial_state: Cartesian,
        osculating: bool = False,
        step: float | None = None,
    ) -> Self: ...
    @overload
    def propagate(self, steps: Time) -> Cartesian: ...
    @overload
    def propagate(self, steps: list[Time]) -> Trajectory: ...
    def propagate(
        self,
        steps: Time | list[Time],
        end: Time | None = None,
        frame: str | Frame | None = None,
        provider: EOPProvider | None = None,
    ) -> Cartesian | Trajectory:
        """Propagate the orbit to one or more times."""
        ...

class GroundLocation:
    """Represents a location on the surface of a celestial body.

    Args:
        origin: The central body (e.g., Earth, Moon).
        longitude: Geodetic longitude as Angle.
        latitude: Geodetic latitude as Angle.
        altitude: Altitude above the reference ellipsoid as Distance.

    Examples:
        >>> darmstadt = lox.GroundLocation(
        ...     lox.Origin("Earth"),
        ...     longitude=8.6512 * lox.deg,
        ...     latitude=49.8728 * lox.deg,
        ...     altitude=0.108 * lox.km,
        ... )
    """
    def __new__(
        cls,
        origin: str | int | Origin,
        longitude: Angle,
        latitude: Angle,
        altitude: Distance,
    ) -> Self: ...
    def longitude(self) -> Angle:
        """Return the geodetic longitude."""
        ...
    def latitude(self) -> Angle:
        """Return the geodetic latitude."""
        ...
    def altitude(self) -> Distance:
        """Return the altitude above the reference ellipsoid."""
        ...
    def observables(
        self,
        state: Cartesian,
        provider: EOPProvider | None = None,
        frame: str | Frame | None = None,
    ) -> Observables:
        """Compute observables to a target state."""
        ...
    def origin(self) -> Origin:
        """Return the central body (origin)."""
        ...
    def rotation_to_topocentric(self) -> np.ndarray:
        """Return the rotation matrix from body-fixed to topocentric frame."""
        ...

class GroundPropagator:
    """Propagator for ground station positions.

    Args:
        location: The ground location to propagate.

    Examples:
        >>> gs = lox.GroundLocation(lox.Origin("Earth"), lon, lat, alt)
        >>> prop = lox.GroundPropagator(gs)
        >>> trajectory = prop.propagate([t1, t2, t3])
    """
    def __new__(cls, location: GroundLocation) -> Self: ...
    @overload
    def propagate(self, steps: Time) -> Cartesian: ...
    @overload
    def propagate(self, steps: list[Time]) -> Trajectory: ...
    def propagate(
        self,
        steps: Time | list[Time],
        end: Time | None = None,
        frame: str | Frame | None = None,
        provider: EOPProvider | None = None,
    ) -> Cartesian | Trajectory:
        """Propagate the ground station to one or more times."""
        ...

class TLE:
    """Two-Line Element set (TLE) for satellite orbit data.

    Parses and exposes the orbital elements from a NORAD Two-Line Element set.

    Args:
        tle: TLE as a string (2 or 3 lines) or a list of 2–3 strings.
    """
    def __new__(cls, tle: str | list[str]) -> Self: ...
    def object_name(self) -> str | None:
        """Satellite name, if present (line 0 of a 3-line TLE)."""
        ...
    def international_designator(self) -> str | None:
        """International designator (e.g. '98067A')."""
        ...
    def norad_id(self) -> int:
        """NORAD catalog number."""
        ...
    def classification(self) -> str:
        """Classification: 'U' (unclassified), 'C' (classified), or 'S' (secret)."""
        ...
    def epoch(self) -> Time:
        """TLE epoch as a Time (TAI scale)."""
        ...
    def inclination(self) -> Angle:
        """Orbital inclination."""
        ...
    def right_ascension(self) -> Angle:
        """Right ascension of the ascending node (RAAN)."""
        ...
    def eccentricity(self) -> float:
        """Orbital eccentricity (dimensionless)."""
        ...
    def argument_of_perigee(self) -> Angle:
        """Argument of perigee."""
        ...
    def mean_anomaly(self) -> Angle:
        """Mean anomaly."""
        ...
    def mean_motion(self) -> float:
        """Mean motion in revolutions per day (Kozai convention)."""
        ...
    def mean_motion_dot(self) -> float:
        """First derivative of mean motion (rev/day²)."""
        ...
    def mean_motion_ddot(self) -> float:
        """Second derivative of mean motion (rev/day³)."""
        ...
    def drag_term(self) -> float:
        """BSTAR drag term (earth radii⁻¹)."""
        ...
    def element_set_number(self) -> int:
        """Element set number."""
        ...
    def revolution_number(self) -> int:
        """Revolution number at epoch."""
        ...
    def ephemeris_type(self) -> int:
        """Ephemeris type (always 0 in distributed data)."""
        ...
    def __repr__(self) -> str: ...
    def __str__(self) -> str: ...
    def __getnewargs__(self) -> tuple[str]: ...

class SGP4:
    """SGP4 (Simplified General Perturbations 4) orbit propagator.

    SGP4 is the standard propagator for objects tracked by NORAD/Space-Track.
    It uses Two-Line Element (TLE) data.

    Args:
        tle: TLE object, string (2 or 3 lines), or list of 2–3 strings.

    Examples:
        >>> tle = '''ISS (ZARYA)
        ... 1 25544U 98067A   24001.50000000  .00016717  00000-0  10270-3 0  9002
        ... 2 25544  51.6400 208.9163 0006703  40.7490  46.4328 15.49952307    11'''
        >>> sgp4 = lox.SGP4(tle)
        >>> trajectory = sgp4.propagate([t1, t2, t3])
    """
    def __new__(cls, tle: TLE | str | list[str]) -> Self: ...
    def tle(self) -> TLE:
        """Return the parsed TLE."""
        ...
    def time(self) -> Time:
        """Return the TLE epoch time."""
        ...
    @overload
    def propagate(
        self, steps: Time, provider: EOPProvider | None = None
    ) -> Cartesian: ...
    @overload
    def propagate(
        self, steps: list[Time], provider: EOPProvider | None = None
    ) -> Trajectory: ...
    def propagate(
        self,
        steps: Time | list[Time],
        end: Time | None = None,
        frame: str | Frame | None = None,
        provider: EOPProvider | None = None,
    ) -> Cartesian | Trajectory:
        """Propagate the orbit to one or more times."""
        ...

class Observables:
    """Observation data from a ground station to a target.

    Args:
        azimuth: Azimuth angle as Angle.
        elevation: Elevation angle as Angle.
        range: Distance to target as Distance.
        range_rate: Rate of change of range as Velocity.
    """
    def __new__(
        cls, azimuth: Angle, elevation: Angle, range: Distance, range_rate: Velocity
    ) -> Self: ...
    def azimuth(self) -> Angle:
        """Return the azimuth angle."""
        ...
    def elevation(self) -> Angle:
        """Return the elevation angle."""
        ...
    def range(self) -> Distance:
        """Return the range (distance)."""
        ...
    def range_rate(self) -> Velocity:
        """Return the range rate."""
        ...

class Pass:
    """Represents a visibility pass between a ground station and spacecraft.

    A Pass contains the visibility interval (start and end times) along with
    observables computed at regular intervals throughout the pass.
    """
    def __new__(
        cls, interval: Interval, times: list[Time], observables: list[Observables]
    ) -> Self: ...
    def interval(self) -> Interval:
        """Return the visibility interval for this pass."""
        ...
    def times(self) -> list[Time]:
        """Return the time samples during this pass."""
        ...
    def observables(self) -> list[Observables]:
        """Return the observables at each time sample."""
        ...
    def interpolate(self, time: Time) -> Observables | None:
        """Interpolate observables at a specific time within the pass."""
        ...
    def __repr__(self) -> str: ...

class TimeScale:
    """Represents an astronomical time scale.

    Supported time scales:
    - TAI: International Atomic Time
    - TT: Terrestrial Time
    - TDB: Barycentric Dynamical Time
    - TCB: Barycentric Coordinate Time
    - TCG: Geocentric Coordinate Time
    - UT1: Universal Time (requires EOP data)

    Args:
        abbreviation: Time scale abbreviation.

    Examples:
        >>> tai = lox.TimeScale("TAI")
        >>> tai.name()
        'International Atomic Time'
    """
    def __new__(cls, abbreviation: Scale) -> Self: ...
    def abbreviation(self) -> str:
        """Return the time scale abbreviation."""
        ...
    def name(self) -> str:
        """Return the full name of the time scale."""
        ...

class Time:
    """Represents an instant in time on a specific astronomical time scale.

    Time provides femtosecond precision and support for multiple astronomical
    time scales (TAI, TT, TDB, TCB, TCG, UT1).

    Args:
        scale: Time scale ("TAI", "TT", etc.) or TimeScale object.
        year: Calendar year.
        month: Calendar month (1-12).
        day: Day of month (1-31).
        hour: Hour (0-23, default 0).
        minute: Minute (0-59, default 0).
        seconds: Seconds including fractional part (default 0.0).

    Examples:
        >>> t = lox.Time("TAI", 2024, 1, 1, 12, 0, 0.0)
        >>> t.to_scale("TT")
        Time(TT, 2024, 1, 1, 12, 0, 32.184)
    """
    def __new__(
        cls,
        scale: Scale | TimeScale,
        year: int,
        month: int,
        day: int,
        hour: int = 0,
        minute: int = 0,
        seconds: float = 0.0,
    ) -> Self: ...
    @classmethod
    def from_julian_date(
        cls, scale: Scale | TimeScale, jd: float, epoch: str = "jd"
    ) -> Self:
        """Create a Time from a Julian date."""
        ...
    @classmethod
    def from_two_part_julian_date(
        cls, scale: Scale | TimeScale, jd1: float, jd2: float
    ) -> Self:
        """Create a Time from a two-part Julian date for maximum precision."""
        ...
    @classmethod
    def from_day_of_year(
        cls,
        scale: Scale | TimeScale,
        year: int,
        doy: int,
        hour: int = 0,
        minute: int = 0,
        seconds: float = 0.0,
    ) -> Self:
        """Create a Time from a day-of-year representation."""
        ...
    @classmethod
    def from_iso(cls, iso: str, scale: Scale | TimeScale | None = None) -> Self:
        """Parse a Time from an ISO 8601 string."""
        ...
    @classmethod
    def from_seconds(
        cls, scale: Scale | TimeScale, seconds: int, subsecond: float
    ) -> Self:
        """Create a Time from seconds since J2000."""
        ...
    def seconds(self) -> int:
        """Return integer seconds since J2000."""
        ...
    def subsecond(self) -> float:
        """Return the fractional part of the second."""
        ...
    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    def __add__(self, other: TimeDelta) -> Self: ...
    @overload
    def __sub__(self, other: TimeDelta) -> Self: ...
    @overload
    def __sub__(self, other: Time) -> TimeDelta: ...
    def __eq__(self, other: object) -> bool: ...
    def __lt__(self, other: object) -> bool: ...
    def __le__(self, other: object) -> bool: ...
    def isclose(
        self, other: Time, rel_tol: float = 1e-8, abs_tol: float = 1e-14
    ) -> bool:
        """Check if two times are approximately equal."""
        ...
    def julian_date(
        self,
        epoch: Epoch = "jd",
        unit: Unit = "days",
    ) -> float:
        """Return the Julian date."""
        ...
    def two_part_julian_date(self) -> tuple[float, float]:
        """Return the two-part Julian date for maximum precision."""
        ...
    def scale(self) -> TimeScale:
        """Return the time scale of this time."""
        ...
    def year(self) -> int:
        """Return the calendar year."""
        ...
    def month(self) -> int:
        """Return the calendar month (1-12)."""
        ...
    def day(self) -> int:
        """Return the day of month (1-31)."""
        ...
    def day_of_year(self) -> int:
        """Return the day of year (1-366)."""
        ...
    def hour(self) -> int:
        """Return the hour (0-23)."""
        ...
    def minute(self) -> int:
        """Return the minute (0-59)."""
        ...
    def second(self) -> int:
        """Return the integer second (0-59)."""
        ...
    def millisecond(self) -> int:
        """Return the millisecond component (0-999)."""
        ...
    def microsecond(self) -> int:
        """Return the microsecond component (0-999)."""
        ...
    def nanosecond(self) -> int:
        """Return the nanosecond component (0-999)."""
        ...
    def picosecond(self) -> int:
        """Return the picosecond component (0-999)."""
        ...
    def femtosecond(self) -> int:
        """Return the femtosecond component (0-999)."""
        ...
    def decimal_seconds(self) -> float:
        """Return the decimal seconds within the current minute."""
        ...
    def to_scale(
        self, scale: Scale | TimeScale, provider: EOPProvider | None = None
    ) -> Self:
        """Convert to a different time scale."""
        ...
    def to_utc(self, provider: EOPProvider | None = None) -> UTC:
        """Convert to UTC."""
        ...

class TimeDelta:
    """Represents a duration or time difference.

    TimeDelta represents a time interval with femtosecond precision.

    Args:
        seconds: Duration in seconds.

    Examples:
        >>> dt = lox.TimeDelta(3600.0)  # 1 hour
        >>> dt = lox.TimeDelta.from_hours(1.0)
        >>> t2 = t1 + dt
    """
    def __new__(cls, seconds: float) -> Self: ...
    def __repr__(self) -> str: ...
    def __str__(self) -> str: ...
    def __float__(self) -> float: ...
    def __neg__(self) -> Self: ...
    def __add__(self, other: Self) -> Self: ...
    def __sub__(self, other: Self) -> Self: ...
    def __mul__(self, other: float) -> Self: ...
    def __rmul__(self, other: float) -> Self: ...
    def seconds(self) -> int:
        """Return integer seconds."""
        ...
    def subsecond(self) -> float:
        """Return the fractional part of the second."""
        ...
    @classmethod
    def from_seconds(cls, seconds: int) -> Self:
        """Create from integer seconds."""
        ...
    @classmethod
    def from_minutes(cls, minutes: float) -> Self:
        """Create from minutes."""
        ...
    @classmethod
    def from_hours(cls, hours: float) -> Self:
        """Create from hours."""
        ...
    @classmethod
    def from_days(cls, days: float) -> Self:
        """Create from days."""
        ...
    @classmethod
    def from_julian_years(cls, years: float) -> Self:
        """Create from Julian years (365.25 days)."""
        ...
    @classmethod
    def from_julian_centuries(cls, centuries: float) -> Self:
        """Create from Julian centuries (36525 days)."""
        ...
    @classmethod
    def from_milliseconds(cls, ms: int) -> Self:
        """Create from integer milliseconds."""
        ...
    @classmethod
    def from_microseconds(cls, us: int) -> Self:
        """Create from integer microseconds."""
        ...
    @classmethod
    def from_nanoseconds(cls, ns: int) -> Self:
        """Create from integer nanoseconds."""
        ...
    @classmethod
    def from_picoseconds(cls, ps: int) -> Self:
        """Create from integer picoseconds."""
        ...
    @classmethod
    def from_femtoseconds(cls, fs: int) -> Self:
        """Create from integer femtoseconds."""
        ...
    @classmethod
    def from_attoseconds(cls, atto: int) -> Self:
        """Create from integer attoseconds."""
        ...
    def to_decimal_seconds(self) -> float:
        """Return the duration as decimal seconds."""
        ...
    @classmethod
    def range(cls, start: int, end: int, step: int | None = None) -> list[Self]:
        """Generate a range of TimeDelta values."""
        ...

class UTC:
    """Represents a UTC (Coordinated Universal Time) timestamp.

    UTC is the basis for civil time worldwide. Unlike Time, UTC handles
    leap seconds and is discontinuous.

    Args:
        year: Calendar year.
        month: Calendar month (1-12).
        day: Day of month (1-31).
        hour: Hour (0-23, default 0).
        minute: Minute (0-59, default 0).
        seconds: Seconds including fractional part (default 0.0).

    Examples:
        >>> utc = lox.UTC(2024, 1, 1, 12, 0, 0.0)
        >>> t = utc.to_scale("TAI")
    """
    def __new__(
        cls,
        year: int,
        month: int,
        day: int,
        hour: int = 0,
        minute: int = 0,
        seconds: float = 0.0,
    ) -> Self: ...
    @classmethod
    def from_iso(cls, iso: str) -> Self:
        """Parse from an ISO 8601 string."""
        ...
    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    def __eq__(self, other: object) -> bool: ...
    def isclose(
        self, other: UTC, rel_tol: float = 1e-8, abs_tol: float = 1e-14
    ) -> bool:
        """Check if two UTC times are approximately equal."""
        ...
    def year(self) -> int:
        """Return the calendar year."""
        ...
    def month(self) -> int:
        """Return the calendar month (1-12)."""
        ...
    def day(self) -> int:
        """Return the day of month (1-31)."""
        ...
    def hour(self) -> int:
        """Return the hour (0-23)."""
        ...
    def minute(self) -> int:
        """Return the minute (0-59)."""
        ...
    def second(self) -> int:
        """Return the integer second (0-60, 60 for leap seconds)."""
        ...
    def millisecond(self) -> int:
        """Return the millisecond component (0-999)."""
        ...
    def microsecond(self) -> int:
        """Return the microsecond component (0-999)."""
        ...
    def nanosecond(self) -> int:
        """Return the nanosecond component (0-999)."""
        ...
    def picosecond(self) -> int:
        """Return the picosecond component (0-999)."""
        ...
    def decimal_seconds(self) -> float:
        """Return the decimal seconds within the current minute."""
        ...
    def to_scale(
        self, scale: Scale | TimeScale, provider: EOPProvider | None = None
    ) -> Time:
        """Convert to a continuous time scale."""
        ...

class EOPProvider:
    """Earth Orientation Parameters (EOP) data provider.

    EOP data is required for accurate transformations involving UT1 and
    polar motion corrections.

    Args:
        *args: Path(s) to EOP data file(s) (CSV format).

    Raises:
        EopParserError: If the file cannot be parsed.
        OSError: If the file cannot be read.

    Examples:
        >>> eop = lox.EOPProvider("/path/to/finals2000A.all.csv")
        >>> t_ut1 = t_tai.to_scale("UT1", provider=eop)
    """
    def __new__(cls, *args: os.PathLike) -> Self: ...

class Series:
    """Interpolation series for 1D data.

    Args:
        x: Array of x values (must be monotonically increasing).
        y: Array of y values (same length as x).
        method: Interpolation method ("linear" or "cubic").

    Examples:
        >>> x = [0.0, 1.0, 2.0, 3.0]
        >>> y = [0.0, 1.0, 4.0, 9.0]
        >>> series = lox.Series(x, y, method="cubic")
        >>> series.interpolate(1.5)
        2.25
    """
    def __new__(
        cls,
        x: list[float],
        y: list[float],
        interpolation: Literal["linear", "cubic"] = "linear",
    ) -> Self: ...
    def interpolate(self, xp: float) -> float:
        """Interpolate a y value at the given x coordinate."""
        ...

class TimeSeries:
    """Time-indexed interpolation series.

    Wraps a Series with a start epoch, allowing interpolation by Time values
    rather than raw float offsets.

    Args:
        times: List of Time objects (must be in chronological order).
        values: List of y values (same length as times).
        interpolation: Interpolation method ("linear" or "cubic").

    Examples:
        >>> epoch = lox.Time("TAI", 2024, 1, 1)
        >>> times = [epoch, epoch + 60 * lox.seconds, epoch + 120 * lox.seconds]
        >>> ts = lox.TimeSeries(times, [1.0, 2.0, 3.0])
        >>> ts.interpolate(epoch + 30 * lox.seconds)
        1.5
    """
    def __new__(
        cls,
        times: list[Time],
        values: list[float],
        interpolation: Literal["linear", "cubic"] = "linear",
    ) -> Self: ...
    @classmethod
    def from_offsets(
        cls,
        epoch: Time,
        x: list[float],
        y: list[float],
        interpolation: Literal["linear", "cubic"] = "linear",
    ) -> TimeSeries:
        """Create a TimeSeries from an epoch and relative offsets in seconds."""
        ...
    def interpolate(self, time: Time) -> float:
        """Interpolate a y value at the given time."""
        ...
    @property
    def epoch(self) -> Time:
        """The reference epoch of this time series."""
        ...
    def times(self) -> list[Time]:
        """Return absolute timestamps for each data point."""
        ...
    def values(self) -> list[float]:
        """Return the y values."""
        ...
    def first(self) -> tuple[Time, float]:
        """Return the first data point as (time, value)."""
        ...
    def last(self) -> tuple[Time, float]:
        """Return the last data point as (time, value)."""
        ...

# Communications

class Decibel:
    """A value in decibels.

    Args:
        value: The value in dB.
    """
    def __new__(cls, value: float) -> Self: ...
    @staticmethod
    def from_linear(value: float) -> Decibel:
        """Creates a Decibel value from a linear power ratio."""
        ...
    def to_linear(self) -> float:
        """Returns the linear power ratio."""
        ...
    def __float__(self) -> float: ...
    def __add__(self, other: Decibel) -> Decibel: ...
    def __sub__(self, other: Decibel) -> Decibel: ...
    def __mul__(self, other: float) -> Decibel: ...
    def __rmul__(self, other: float) -> Decibel: ...
    def __neg__(self) -> Decibel: ...
    def __eq__(self, other: object) -> bool: ...
    def __repr__(self) -> str: ...
    def __str__(self) -> str: ...

class Modulation:
    """Digital modulation scheme.

    Args:
        name: One of "BPSK", "QPSK", "8PSK", "16QAM", "32QAM", "64QAM", "128QAM", "256QAM".
    """
    def __new__(cls, name: str) -> Self: ...
    def bits_per_symbol(self) -> int:
        """Returns the number of bits per symbol."""
        ...
    def __eq__(self, other: object) -> bool: ...
    def __repr__(self) -> str: ...

class ParabolicPattern:
    """Parabolic antenna gain pattern.

    Args:
        diameter: Antenna diameter as Distance.
        efficiency: Aperture efficiency (0, 1].
    """
    def __new__(cls, diameter: Distance, efficiency: float) -> Self: ...
    @staticmethod
    def from_beamwidth(
        beamwidth: Angle, frequency: Frequency, efficiency: float
    ) -> ParabolicPattern:
        """Creates a parabolic pattern from a desired beamwidth.

        Args:
            beamwidth: Half-power beamwidth as Angle.
            frequency: Frequency.
            efficiency: Aperture efficiency (0, 1].
        """
        ...
    def gain(
        self, frequency: Frequency, theta: Angle, phi: Angle | None = None
    ) -> Decibel:
        """Returns the gain in dBi at the given frequency and pattern angles."""
        ...
    def beamwidth(self, frequency: Frequency) -> Angle | None:
        """Returns the half-power beamwidth, or ``None`` when the
        antenna diameter is smaller than ~0.51 wavelengths at this frequency."""
        ...
    def peak_gain(self, frequency: Frequency) -> Decibel:
        """Returns the peak gain in dBi."""
        ...
    def __eq__(self, other: object) -> bool: ...
    def __repr__(self) -> str: ...

class GaussianPattern:
    """Gaussian antenna gain pattern.

    Args:
        diameter: Antenna diameter as Distance.
        efficiency: Aperture efficiency (0, 1].
    """
    def __new__(cls, diameter: Distance, efficiency: float) -> Self: ...
    def gain(
        self, frequency: Frequency, theta: Angle, phi: Angle | None = None
    ) -> Decibel:
        """Returns the gain in dBi at the given frequency and pattern angles."""
        ...
    def beamwidth(self, frequency: Frequency) -> Angle:
        """Returns the half-power beamwidth."""
        ...
    def peak_gain(self, frequency: Frequency) -> Decibel:
        """Returns the peak gain in dBi."""
        ...
    def __eq__(self, other: object) -> bool: ...
    def __repr__(self) -> str: ...

class DipolePattern:
    """Dipole antenna gain pattern.

    Args:
        length: Dipole length as Distance.
    """
    def __new__(cls, length: Distance) -> Self: ...
    def gain(
        self, frequency: Frequency, theta: Angle, phi: Angle | None = None
    ) -> Decibel:
        """Returns the gain in dBi at the given frequency and pattern angles."""
        ...
    def peak_gain(self, frequency: Frequency) -> Decibel:
        """Returns the peak gain in dBi."""
        ...
    def __eq__(self, other: object) -> bool: ...
    def __repr__(self) -> str: ...

class ConstantAntenna:
    """An antenna with constant gain.

    Args:
        gain: Peak gain as Decibel.
    """
    def __new__(cls, gain: Decibel) -> Self: ...
    def __eq__(self, other: object) -> bool: ...
    def __repr__(self) -> str: ...

class AntennaFrame:
    """Right-handed antenna coordinate frame expressed in a parent frame.

    Args:
        boresight: Antenna +Z axis as [x, y, z].
        reference: Direction used to define the antenna +X axis after projection
            into the plane perpendicular to boresight.
    """
    def __new__(cls, boresight: list[float], reference: list[float]) -> Self: ...
    @staticmethod
    def identity() -> AntennaFrame:
        """Creates an antenna frame aligned with the parent frame."""
        ...
    @staticmethod
    def from_boresight_and_reference(
        boresight: list[float], reference: list[float]
    ) -> AntennaFrame:
        """Creates an antenna frame from boresight and reference directions."""
        ...
    def x(self) -> list[float]:
        """Returns the antenna-frame +X axis in the parent frame."""
        ...
    def y(self) -> list[float]:
        """Returns the antenna-frame +Y axis in the parent frame."""
        ...
    def z(self) -> list[float]:
        """Returns the antenna-frame +Z axis in the parent frame."""
        ...
    def angles_for(self, direction: list[float]) -> tuple[Angle, Angle]:
        """Returns the pattern angles for a parent-frame direction vector."""
        ...
    def __eq__(self, other: object) -> bool: ...
    def __repr__(self) -> str: ...

class PatternedAntenna:
    """An antenna with a physics-based gain pattern and antenna frame.

    Args:
        pattern: An antenna pattern (ParabolicPattern, GaussianPattern, or DipolePattern).
        frame: Antenna frame defining the pattern orientation. Defaults to identity.
    """
    def __new__(
        cls,
        pattern: ParabolicPattern | GaussianPattern | DipolePattern,
        frame: AntennaFrame | None = None,
    ) -> Self: ...
    def gain(
        self, frequency: Frequency, theta: Angle, phi: Angle | None = None
    ) -> Decibel:
        """Returns the gain in dBi at the given frequency and pattern angles."""
        ...
    def peak_gain(self, frequency: Frequency) -> Decibel:
        """Returns the peak gain in dBi."""
        ...
    def gain_toward(self, frequency: Frequency, direction: list[float]) -> Decibel:
        """Returns the gain in dBi toward a parent-frame direction vector."""
        ...
    def beamwidth(self, frequency: Frequency) -> Angle | None:
        """Returns the half-power beamwidth, or ``None`` when the underlying
        pattern does not define one."""
        ...
    def __repr__(self) -> str: ...

class AmplifierTransmitter:
    """A radio transmitter with an RF power amplifier.

    Args:
        band: Supported frequency range.
        power: Transmit power.
        output_back_off: Output back-off as Decibel (default Decibel(0)).
    """
    def __new__(
        cls,
        band: FrequencyRange,
        power: Power,
        output_back_off: Decibel | None = None,
    ) -> Self: ...
    @property
    def band(self) -> FrequencyRange:
        """Supported frequency range."""
        ...
    @property
    def power(self) -> Power:
        """Transmit power."""
        ...
    @property
    def output_back_off(self) -> Decibel:
        """Output back-off."""
        ...
    def __eq__(self, other: object) -> bool: ...
    def __repr__(self) -> str: ...

class NoiseTempReceiver:
    """A receiver characterised by a single equivalent noise temperature.

    The figure is referred to the receiver's input connector; the system
    noise temperature at the antenna flange is assembled at link-budget
    setup from the port's antenna noise temperature and feed loss. For a
    datasheet figure that already includes antenna and feed contributions,
    set both port values to zero.

    Args:
        band: Supported frequency range.
        noise_temperature: Equivalent noise temperature at the input connector.
    """
    def __new__(
        cls, band: FrequencyRange, noise_temperature: Temperature
    ) -> Self: ...
    @property
    def band(self) -> FrequencyRange:
        """Supported frequency range."""
        ...
    @property
    def noise_temperature(self) -> Temperature:
        """Equivalent noise temperature referred to the receiver's input connector."""
        ...
    def __eq__(self, other: object) -> bool: ...
    def __repr__(self) -> str: ...

class NoiseStage:
    """A single stage in an RF receiver chain.

    Args:
        gain: Stage gain as Decibel.
        noise_temperature: Stage equivalent noise temperature.
    """
    def __new__(cls, gain: Decibel, noise_temperature: Temperature) -> Self: ...
    def __repr__(self) -> str: ...

class CascadeReceiver:
    """An N-stage cascade receiver using the Friis noise formula.

    Args:
        band: Supported frequency range.
        stages: List of NoiseStage (ordered: LNA first, then downstream).
        demodulator_loss: Demodulator loss as Decibel (default Decibel(0)).
        implementation_loss: Other implementation losses as Decibel (default Decibel(0)).
    """
    def __new__(
        cls,
        band: FrequencyRange,
        stages: list[NoiseStage],
        demodulator_loss: Decibel | None = None,
        implementation_loss: Decibel | None = None,
    ) -> Self: ...
    @staticmethod
    def from_lna_and_noise_figure(
        band: FrequencyRange,
        lna_gain: Decibel,
        lna_noise_temperature: Temperature,
        receiver_noise_figure: Decibel,
        demodulator_loss: Decibel | None = None,
        implementation_loss: Decibel | None = None,
    ) -> CascadeReceiver:
        """Creates a two-stage model: LNA followed by a receiver characterised by noise figure."""
        ...
    def chain_noise_temperature(self) -> Temperature:
        """Returns the chain's equivalent noise temperature referred to its input connector, via the Friis formula."""
        ...
    def chain_gain(self) -> Decibel:
        """Returns the total RF chain gain in dB."""
        ...
    @property
    def band(self) -> FrequencyRange:
        """Supported frequency range."""
        ...
    def __eq__(self, other: object) -> bool: ...
    def __repr__(self) -> str: ...

class Channel:
    """A communication channel.

    Args:
        link_type: "uplink", "downlink", or "crosslink".
        symbol_rate: Symbol rate as Frequency.
        required_eb_n0: Required Eb/N0 as Decibel.
        margin: Required link margin as Decibel.
        modulation: Modulation scheme.
        roll_off: Roll-off factor (default 0.35).
        fec: Forward error correction code rate (default 0.5).
        chip_rate: Chip rate for DSSS as Frequency (optional).
    """
    def __new__(
        cls,
        link_type: str,
        symbol_rate: Frequency,
        required_eb_n0: Decibel,
        margin: Decibel,
        modulation: Modulation,
        roll_off: float = 0.35,
        fec: float = 0.5,
        chip_rate: Frequency | None = None,
    ) -> Self: ...
    def data_rate(self) -> Frequency:
        """Returns the raw bit rate."""
        ...
    def information_rate(self) -> Frequency:
        """Returns the information (post-FEC) bit rate."""
        ...
    def bandwidth(self) -> Frequency:
        """Returns the occupied channel bandwidth."""
        ...
    def es_n0(self, c_n0: Decibel) -> Decibel:
        """Computes Es/N0 from a given C/N0."""
        ...
    def eb_n0(self, c_n0: Decibel) -> Decibel:
        """Computes Eb/N0 from a given C/N0."""
        ...
    def c_n(self, c_n0: Decibel) -> Decibel:
        """Computes C/N from a given C/N0."""
        ...
    def link_margin(self, eb_n0: Decibel) -> Decibel:
        """Computes the link margin from a given Eb/N0."""
        ...
    def spreading_factor(self) -> float | None:
        """Returns the DSSS spreading factor, or None for narrowband."""
        ...
    def processing_gain(self) -> Decibel | None:
        """Returns the DSSS processing gain in dB, or None for narrowband."""
        ...
    def apply(self, link: LinkStats) -> ModulatedLinkStats:
        """Layers modulation/FEC figures onto a modulation-agnostic link budget."""
        ...
    def __repr__(self) -> str: ...

class EnvironmentalLosses:
    """Atmospheric environmental losses computed from ITU-R models.

    Computes rain, gaseous, cloud, scintillation, and depolarization
    attenuation for a slant path.

    Args:
        provider: Open ItuProvider supplying the gridded reference data.
        lat: Latitude.
        lon: Longitude.
        frequency: Frequency.
        elevation: Elevation angle (clamped to >= 5 deg).
        probability: Exceedance probability (% of average year).
        diameter: Physical antenna diameter.
        polarisation_tilt: Polarisation tilt angle (default 45 deg for circular).
    """
    def __new__(
        cls,
        provider: ItuProvider,
        lat: Angle,
        lon: Angle,
        frequency: Frequency,
        elevation: Angle,
        probability: float,
        diameter: Distance,
        polarisation_tilt: Angle | None = None,
    ) -> Self: ...
    @property
    def rain(self) -> Decibel:
        """Rain attenuation."""
        ...
    @property
    def gaseous(self) -> Decibel:
        """Gaseous absorption."""
        ...
    @property
    def scintillation(self) -> Decibel:
        """Scintillation loss."""
        ...
    @property
    def atmospheric(self) -> Decibel:
        """General atmospheric loss (combined total)."""
        ...
    @property
    def cloud(self) -> Decibel:
        """Cloud attenuation."""
        ...
    @property
    def depolarization(self) -> Decibel:
        """Depolarization loss."""
        ...
    def total(self) -> Decibel:
        """Returns the total environmental loss in dB."""
        ...
    def __eq__(self, other: object) -> bool: ...
    def __repr__(self) -> str: ...

class FrequencyRange:
    """A contiguous frequency range with inclusive bounds.

    Args:
        min: Lower frequency bound.
        max: Upper frequency bound.
    """
    def __new__(cls, min: Frequency, max: Frequency) -> Self: ...
    @staticmethod
    def from_wavelengths(
        min_wavelength: Distance, max_wavelength: Distance
    ) -> FrequencyRange:
        """Creates a frequency range from wavelength bounds (e.g. optical bands)."""
        ...
    def min(self) -> Frequency:
        """Returns the lower frequency bound."""
        ...
    def max(self) -> Frequency:
        """Returns the upper frequency bound."""
        ...
    def contains(self, frequency: Frequency) -> bool:
        """Returns whether the frequency lies within the range (bounds inclusive)."""
        ...
    def __eq__(self, other: object) -> bool: ...
    def __repr__(self) -> str: ...
    def __str__(self) -> str: ...

class AntennaId:
    """Identifier of an antenna in a CommsPayload. Only valid for the payload that minted it."""
    def __eq__(self, other: object) -> bool: ...
    def __repr__(self) -> str: ...

class TransmitterId:
    """Identifier of a transmitter in a CommsPayload. Only valid for the payload that minted it."""
    def __eq__(self, other: object) -> bool: ...
    def __repr__(self) -> str: ...

class ReceiverId:
    """Identifier of a receiver in a CommsPayload. Only valid for the payload that minted it."""
    def __eq__(self, other: object) -> bool: ...
    def __repr__(self) -> str: ...

class EirpModelId:
    """Identifier of a lumped EIRP model in a CommsPayload. Only valid for the payload that minted it."""
    def __eq__(self, other: object) -> bool: ...
    def __repr__(self) -> str: ...

class GtModelId:
    """Identifier of a lumped G/T model in a CommsPayload. Only valid for the payload that minted it."""
    def __eq__(self, other: object) -> bool: ...
    def __repr__(self) -> str: ...

class TxPortId:
    """Identifier of a transmit port in a CommsPayload. Only valid for the payload that minted it."""
    def __eq__(self, other: object) -> bool: ...
    def __repr__(self) -> str: ...

class RxPortId:
    """Identifier of a receive port in a CommsPayload. Only valid for the payload that minted it."""
    def __eq__(self, other: object) -> bool: ...
    def __repr__(self) -> str: ...

class TerminalId:
    """Identifier of a terminal in a CommsPayload. Only valid for the payload that minted it."""
    def __eq__(self, other: object) -> bool: ...
    def __repr__(self) -> str: ...

class CommsPayload:
    """Communications hardware inventory and wiring for one platform.

    Owns antennas, radios, lumped models, ports, and terminals. Wiring is by
    ID and validated at insertion; names are display-only. The payload holds
    no operational state: carrier, bandwidth, modulation, and pointing are
    link-level inputs.
    """
    def __new__(cls) -> Self: ...
    def add_antenna(
        self, name: str, antenna: ConstantAntenna | PatternedAntenna
    ) -> AntennaId:
        """Adds an antenna to the inventory."""
        ...
    def add_transmitter(self, name: str, transmitter: AmplifierTransmitter) -> TransmitterId:
        """Adds a component-tier transmitter to the inventory."""
        ...
    def add_receiver(
        self, name: str, receiver: NoiseTempReceiver | CascadeReceiver
    ) -> ReceiverId:
        """Adds a component-tier receiver to the inventory."""
        ...
    def add_eirp_model(self, name: str, band: FrequencyRange, eirp: Decibel) -> EirpModelId:
        """Adds a lumped EIRP model to the inventory."""
        ...
    def add_gt_model(self, name: str, band: FrequencyRange, gt: Decibel) -> GtModelId:
        """Adds a lumped G/T model to the inventory."""
        ...
    def add_tx_port(
        self,
        name: str,
        antenna: AntennaId,
        transmitter: TransmitterId,
        feed_loss: Decibel,
        band: FrequencyRange | None = None,
    ) -> TxPortId:
        """Adds a transmit port wiring an antenna to a transmitter."""
        ...
    def add_rx_port(
        self,
        name: str,
        antenna: AntennaId,
        receiver: ReceiverId,
        feed_loss: Decibel,
        antenna_noise_temperature: Temperature,
        band: FrequencyRange | None = None,
    ) -> RxPortId:
        """Adds a receive port wiring an antenna to a receiver."""
        ...
    def add_tx_terminal(
        self,
        name: str,
        port: TxPortId | None = None,
        eirp_model: EirpModelId | None = None,
    ) -> TerminalId:
        """Adds a transmit-only terminal from a port or a lumped EIRP model."""
        ...
    def add_rx_terminal(
        self,
        name: str,
        port: RxPortId | None = None,
        gt_model: GtModelId | None = None,
    ) -> TerminalId:
        """Adds a receive-only terminal from a port or a lumped G/T model."""
        ...
    def add_transceiver_terminal(
        self,
        name: str,
        tx_port: TxPortId | None = None,
        rx_port: RxPortId | None = None,
        eirp_model: EirpModelId | None = None,
        gt_model: GtModelId | None = None,
    ) -> TerminalId:
        """Adds a transceiver terminal with one chain per direction."""
        ...
    def find_terminal(self, name: str) -> TerminalId | None:
        """Returns the first terminal with the given name, if any."""
        ...
    def terminals(self) -> list[tuple[TerminalId, str, str]]:
        """Lists all terminals as (id, name, kind) with kind "tx", "rx", or "transceiver"."""
        ...
    def describe(self) -> str:
        """Returns a multi-line wiring summary for inspection."""
        ...
    def tx_band(self, terminal: TerminalId) -> FrequencyRange:
        """Returns the effective transmit frequency range of a terminal."""
        ...
    def rx_band(self, terminal: TerminalId) -> FrequencyRange:
        """Returns the effective receive frequency range of a terminal."""
        ...
    def eirp_at(
        self,
        terminal: TerminalId,
        carrier: Frequency,
        angle: Angle | None = None,
        direction: list[float] | None = None,
    ) -> Decibel:
        """Returns the EIRP in dBW of a terminal at the given carrier and pointing."""
        ...
    def gt_at(
        self,
        terminal: TerminalId,
        carrier: Frequency,
        angle: Angle | None = None,
        direction: list[float] | None = None,
    ) -> Decibel:
        """Returns the G/T in dB/K of a terminal at the given carrier and pointing."""
        ...
    def __str__(self) -> str: ...
    @staticmethod
    def transmitter_only(
        name: str,
        antenna: ConstantAntenna | PatternedAntenna,
        transmitter: AmplifierTransmitter,
        feed_loss: Decibel,
        band: FrequencyRange | None = None,
    ) -> tuple[CommsPayload, TerminalId]:
        """Creates a single-terminal transmit-only payload."""
        ...
    @staticmethod
    def receiver_only(
        name: str,
        antenna: ConstantAntenna | PatternedAntenna,
        receiver: NoiseTempReceiver | CascadeReceiver,
        feed_loss: Decibel,
        antenna_noise_temperature: Temperature,
        band: FrequencyRange | None = None,
    ) -> tuple[CommsPayload, TerminalId]:
        """Creates a single-terminal receive-only payload."""
        ...
    @staticmethod
    def transceiver(
        name: str,
        antenna: ConstantAntenna | PatternedAntenna,
        transmitter: AmplifierTransmitter,
        receiver: NoiseTempReceiver | CascadeReceiver,
        tx_feed_loss: Decibel,
        rx_feed_loss: Decibel,
        antenna_noise_temperature: Temperature,
        band: FrequencyRange | None = None,
    ) -> tuple[CommsPayload, TerminalId]:
        """Creates a single-terminal transceiver payload sharing one antenna."""
        ...
    @staticmethod
    def eirp_only(name: str, band: FrequencyRange, eirp: Decibel) -> tuple[CommsPayload, TerminalId]:
        """Creates a single-terminal payload from a lumped EIRP model."""
        ...
    @staticmethod
    def gt_only(name: str, band: FrequencyRange, gt: Decibel) -> tuple[CommsPayload, TerminalId]:
        """Creates a single-terminal payload from a lumped G/T model."""
        ...
    def __repr__(self) -> str: ...

class LinkStats:
    """Modulation-agnostic link budget statistics."""
    @staticmethod
    def for_link(
        tx_payload: CommsPayload,
        tx_terminal: TerminalId,
        rx_payload: CommsPayload,
        rx_terminal: TerminalId,
        carrier: Frequency,
        bandwidth: Frequency,
        range: Distance,
        direction: str,
        tx_angle: Angle | None = None,
        rx_angle: Angle | None = None,
        tx_direction: list[float] | None = None,
        rx_direction: list[float] | None = None,
        losses: EnvironmentalLosses | None = None,
    ) -> LinkStats:
        """Computes a modulation-agnostic link budget between payload terminals.

        Resolves the TX and RX terminals into endpoints and evaluates the link
        at the given carrier. The carrier must lie inside both endpoints'
        supported frequency ranges. Each endpoint's pointing is given either
        as an off-boresight angle or as a line-of-sight direction vector in
        the antenna's parent frame; omitting both assumes ideal (boresight)
        pointing. ``direction`` is "uplink", "downlink", or "crosslink".
        """
        ...
    @property
    def slant_range(self) -> Distance:
        """Slant range."""
        ...
    @property
    def fspl(self) -> Decibel:
        """Free-space path loss in dB."""
        ...
    @property
    def eirp(self) -> Decibel:
        """EIRP in dBW."""
        ...
    @property
    def gt(self) -> Decibel:
        """Receiver G/T in dB/K."""
        ...
    @property
    def c_n0(self) -> Decibel:
        """Carrier-to-noise density ratio in dB·Hz."""
        ...
    @property
    def c_n(self) -> Decibel:
        """C/N in dB."""
        ...
    @property
    def carrier_rx_power(self) -> Decibel | None:
        """Received carrier power in dBW. ``None`` for lumped G/T receivers."""
        ...
    @property
    def noise_power(self) -> Decibel | None:
        """Noise power in dBW. ``None`` for lumped G/T receivers."""
        ...
    @property
    def bandwidth(self) -> Frequency:
        """Channel noise bandwidth."""
        ...
    @property
    def frequency(self) -> Frequency:
        """Link frequency."""
        ...
    @property
    def tx_theta(self) -> Angle:
        """Derived TX pattern polar angle from boresight."""
        ...
    @property
    def tx_phi(self) -> Angle:
        """Derived TX pattern azimuth about boresight."""
        ...
    @property
    def rx_theta(self) -> Angle:
        """Derived RX pattern polar angle from boresight."""
        ...
    @property
    def rx_phi(self) -> Angle:
        """Derived RX pattern azimuth about boresight."""
        ...
    @property
    def direction(self) -> str:
        """Link direction ("uplink", "downlink", or "crosslink")."""
        ...
    def __repr__(self) -> str: ...

class InterferenceStats:
    """Interference statistics for a link with a given interferer power."""
    @property
    def interference_power(self) -> Power:
        """Interference power."""
        ...
    @property
    def c_n0i0(self) -> Decibel:
        """Carrier-to-noise-plus-interference density ratio in dB·Hz."""
        ...
    @property
    def eb_n0i0(self) -> Decibel:
        """Eb/(N0+I0) in dB."""
        ...
    @property
    def margin_with_interference(self) -> Decibel:
        """Link margin with interference in dB."""
        ...
    def __repr__(self) -> str: ...

class ModulatedLinkStats:
    """Link-budget output with modulation/coding figures applied."""
    @property
    def link(self) -> LinkStats:
        """The underlying modulation-agnostic link budget."""
        ...
    @property
    def channel(self) -> Channel:
        """The channel (modulation, FEC, required Eb/N0, margin) applied."""
        ...
    @property
    def symbol_rate(self) -> Frequency:
        """Symbol rate from the channel."""
        ...
    @property
    def es_n0(self) -> Decibel:
        """Es/N0 (energy per symbol to noise spectral density) in dB."""
        ...
    @property
    def eb_n0(self) -> Decibel:
        """Eb/N0 (energy per information bit to noise spectral density) in dB."""
        ...
    @property
    def margin(self) -> Decibel:
        """Link margin in dB."""
        ...
    def with_interference(self, interference_power: Power) -> InterferenceStats:
        """Computes interference statistics for a given interferer power."""
        ...
    def __repr__(self) -> str: ...

def fspl(distance: Distance, frequency: Frequency) -> Decibel:
    """Computes the free-space path loss in dB.

    Args:
        distance: Distance.
        frequency: Frequency.

    Returns:
        Free-space path loss as a Decibel value.
    """
    ...

def freq_overlap(
    rx_freq: Frequency, rx_bw: Frequency, tx_freq: Frequency, tx_bw: Frequency
) -> float:
    """Computes the frequency overlap factor between a receiver and an interferer.

    Args:
        rx_freq: Receiver center frequency.
        rx_bw: Receiver bandwidth.
        tx_freq: Interferer center frequency.
        tx_bw: Interferer bandwidth.

    Returns:
        Overlap factor in [0, 1].
    """
    ...

def power_flux_density(
    eirp: Decibel, distance: Distance, occupied_bw: Frequency, reference_bw: Frequency
) -> Decibel:
    """Computes the power flux density in dBW/m²/ref_bw.

    Args:
        eirp: EIRP as Decibel.
        distance: Distance.
        occupied_bw: Occupied bandwidth as Frequency.
        reference_bw: ITU reference bandwidth as Frequency.

    Returns:
        PFD as Decibel.
    """
    ...

class PfdMask:
    """A piecewise-linear PFD mask over elevation in dBW/m²/ref_bw.

    The mask is linear in elevation between consecutive breakpoints and constant
    below the first and above the last.

    Args:
        nodes: (elevation, value) breakpoints with strictly ascending elevations.
    """
    def __new__(cls, nodes: list[tuple[Angle, Decibel]]) -> Self: ...
    @staticmethod
    def art_21_16(start: Decibel) -> PfdMask:
        """The ITU RR Article 21.16 mask shape for a given low-elevation limit.

        Rises from `start` at 5° elevation by 0.5 dB per degree to `start + 10 dB`
        at 25° and is constant outside that range.
        """
        ...
    def value_at(self, elevation: Angle) -> Decibel:
        """Returns the mask value at the given elevation angle."""
        ...
    def nodes(self) -> list[tuple[Angle, Decibel]]:
        """Returns the mask breakpoints as (elevation, value) tuples."""
        ...
    def __eq__(self, other: object) -> bool: ...
    def __repr__(self) -> str: ...

def slant_range(elevation: Angle, earth_radius: Distance, altitude: Distance) -> Distance:
    """Computes the slant range from a ground station to a satellite.

    Args:
        elevation: Elevation angle.
        earth_radius: Earth radius as Distance.
        altitude: Satellite altitude as Distance.

    Returns:
        Slant range as Distance.
    """
    ...

# ---------------------------------------------------------------------------
# Imaging analysis
# ---------------------------------------------------------------------------

class Aoi:
    """An area of interest (AOI) defined as a geographic polygon.

    Coordinates follow GeoJSON convention: longitude/latitude in degrees.

    Args:
        coords: List of (longitude, latitude) tuples forming the polygon
            exterior ring. The ring should be closed (first == last).

    Examples:
        >>> aoi = lox.Aoi([(10, 45), (11, 45), (11, 46), (10, 46), (10, 45)])
        >>> aoi = lox.Aoi.from_geojson('{"type":"Polygon","coordinates":[[[10,45],[11,45],[11,46],[10,46],[10,45]]]}')
    """
    def __new__(cls, coords: list[tuple[float, float]]) -> Self: ...
    @classmethod
    def from_geojson(cls, geojson: str) -> Self:
        """Parse an AOI from a GeoJSON string.

        Expects a GeoJSON Polygon geometry, Feature, or FeatureCollection.
        """
        ...

class OpticalPayload:
    """Optical sensor payload describing a spacecraft's ground coverage capability.

    Defines the sensor's swath width and optional off-nadir pointing capability.
    Assign to a spacecraft via the ``optical_payload`` parameter.

    Examples:
        >>> payload = lox.OpticalPayload.nadir_only(20.0 * lox.km)
        >>> payload = lox.OpticalPayload.off_nadir(20.0 * lox.km, 30.0 * lox.deg)
        >>> sc = lox.Spacecraft("sat1", orbit, optical_payload=payload)
    """
    @classmethod
    def nadir_only(cls, swath_width: Distance) -> Self:
        """Create a payload for a nadir-only sensor.

        Args:
            swath_width: Full swath width as Distance.
        """
        ...
    @classmethod
    def off_nadir(cls, swath_width: Distance, max_off_nadir: Angle) -> Self:
        """Create a payload for a sensor with off-nadir pointing capability.

        Args:
            swath_width: Full swath width as Distance.
            max_off_nadir: Maximum off-nadir angle as Angle.
        """
        ...

class OpticalAccessAnalysis:
    """AOI optical access analysis: computes imaging windows for spacecraft over AOIs.

    Optical payloads are read from each spacecraft; spacecraft without a
    payload are silently skipped.

    Args:
        scenario: Scenario containing spacecraft with ``optical_payload``.
        aois: List of (id, Aoi) tuples defining the areas of interest.
        ensemble: Optional pre-computed Ensemble.
        step: Optional time step for event detection (default: 60s).
        body_fixed_frame: Optional body-fixed frame override.

    Examples:
        >>> payload = lox.OpticalPayload.off_nadir(20.0 * lox.km, 30.0 * lox.deg)
        >>> sc = lox.Spacecraft("sat1", orbit, optical_payload=payload)
        >>> scenario = lox.Scenario(t0, t1, spacecraft=[sc])
        >>> aoi = lox.Aoi.from_geojson('{"type":"Polygon","coordinates":[[[10,45],[11,45],[11,46],[10,46],[10,45]]]}')
        >>> analysis = lox.OpticalAccessAnalysis(scenario, aois=[("rome", aoi)])
        >>> results = analysis.compute()
    """
    def __new__(
        cls,
        scenario: Scenario,
        aois: list[tuple[str, Aoi]],
        ensemble: Ensemble | None = None,
        step: TimeDelta | None = None,
        body_fixed_frame: Frame | None = None,
    ) -> Self: ...
    def compute(self) -> "AccessResults":
        """Compute access intervals for all (spacecraft, AOI) pairs.

        If no ensemble was provided, the scenario is propagated automatically.
        Spacecraft without an optical payload are skipped.
        """
        ...

class AccessResults:
    """Results of an access analysis.

    Provides access windows for each (spacecraft, AOI) pair.
    """
    def windows(self, spacecraft_id: str, aoi_id: str) -> list[AccessWindow]:
        """Return access windows for a specific (spacecraft, AOI) pair.

        Args:
            spacecraft_id: Spacecraft identifier.
            aoi_id: AOI identifier.

        Returns:
            List of AccessWindow objects, or empty list if pair not found.
        """
        ...
    def all_windows(self) -> dict[tuple[str, str], list[AccessWindow]]:
        """Return all access windows for all (spacecraft, AOI) pairs."""
        ...


class PassDirection:
    """Direction of orbital motion at the time of an access window: moving
    northward (``Ascending``) or southward (``Descending``), sampled at the
    window midpoint.
    """

    Ascending: "PassDirection"
    Descending: "PassDirection"


class AccessWindow:
    """A single access window: time interval plus pass direction at the midpoint.

    Examples:
        >>> import lox_space as lox
        >>> results = analysis.compute()
        >>> for window in results.windows("s1a", "europe"):
        ...     print(window.interval(), window.direction())
    """

    def interval(self) -> Interval:
        """Return the access time interval."""
        ...
    def direction(self) -> PassDirection:
        """Return the spacecraft pass direction at the interval midpoint."""
        ...
    def __repr__(self) -> str: ...


class LookSide:
    """Which side of the ground track a SAR payload can image.

    ``Left`` and ``Right`` are defined relative to the spacecraft's instantaneous
    Earth-fixed velocity at the sub-satellite point.
    """

    Left: "LookSide"
    Right: "LookSide"
    Either: "LookSide"


class SarPayload:
    """SAR (Synthetic Aperture Radar) payload — side-looking annular access geometry.

    Construct via :meth:`with_look_angles` (look angle at the satellite) or
    :meth:`with_incidence_angles` (incidence angle at the ground point).

    Assign to a spacecraft via the ``sar_payload`` parameter.

    Examples:
        >>> import lox_space as lox
        >>> payload = lox.SarPayload.with_incidence_angles(29.0 * lox.deg, 46.0 * lox.deg, lox.LookSide.Right)
        >>> sc = lox.Spacecraft("s1a", orbit, sar_payload=payload)
    """

    @classmethod
    def with_look_angles(cls, min: "Angle", max: "Angle", side: LookSide) -> "SarPayload": ...
    @classmethod
    def with_incidence_angles(cls, min: "Angle", max: "Angle", side: LookSide) -> "SarPayload": ...
    def side(self) -> LookSide: ...


class SarAccessAnalysis:
    """SAR access analysis: per-(spacecraft, AOI) access windows.

    Only spacecraft carrying a ``sar_payload`` contribute.

    Args:
        scenario: Scenario containing spacecraft with ``sar_payload``.
        aois: List of (id, Aoi) tuples defining the areas of interest.
        ensemble: Optional pre-computed Ensemble.
        step: Optional time step for event detection (default: 60s).
        body_fixed_frame: Optional body-fixed frame override.

    Examples:
        >>> import lox_space as lox
        >>> payload = lox.SarPayload.with_incidence_angles(29.0 * lox.deg, 46.0 * lox.deg, lox.LookSide.Right)
        >>> sc = lox.Spacecraft("s1a", orbit, sar_payload=payload)
        >>> scenario = lox.Scenario(t0, t1, spacecraft=[sc])
        >>> aoi = lox.Aoi([(10, 45), (11, 45), (11, 46), (10, 46), (10, 45)])
        >>> analysis = lox.SarAccessAnalysis(scenario, aois=[("rome", aoi)])
        >>> results = analysis.compute()
    """

    def __new__(
        cls,
        scenario: Scenario,
        aois: list[tuple[str, Aoi]],
        ensemble: Ensemble | None = None,
        step: TimeDelta | None = None,
        body_fixed_frame: Frame | None = None,
    ) -> Self: ...
    def compute(self) -> "AccessResults":
        """Compute access intervals for all (spacecraft, AOI) pairs.

        If no ensemble was provided, the scenario is propagated automatically.
        Spacecraft without a SAR payload are skipped.
        """
        ...

# ITU-R atmospheric propagation

class ItuProvider:
    """An open ITU-R data bundle (``lox-itur-data.npz``).

    Grid-based recommendations (rain, cloud, scintillation, topography, ...)
    are served as methods on this object. Build a bundle once with::

        cargo run -p lox-itur --bin pack -- <itur-wheel.whl> lox-itur-data.npz
    """
    def __init__(self, path: str) -> None: ...
    def upstream_version(self) -> str:
        """The upstream ``itur`` package version this bundle was built from."""
        ...
    def topographic_altitude(self, lat: Angle, lon: Angle) -> Distance:
        """Return topographic altitude at the given location (P.1511-2)."""
        ...
    def surface_mean_temperature(self, lat: Angle, lon: Angle) -> Temperature:
        """Return annual mean surface temperature at the given location (P.1510-1)."""
        ...
    def rain_height(self, lat: Angle, lon: Angle) -> Distance:
        """Return mean annual rain height at the given location (P.839-4)."""
        ...
    def rainfall_rate(self, lat: Angle, lon: Angle, probability: float) -> float:
        """Return rainfall rate in mm/h exceeded for a given probability (P.837-7)."""
        ...
    def rain_attenuation(
        self,
        lat: Angle,
        lon: Angle,
        frequency: Frequency,
        elevation: Angle,
        probability: float,
        polarisation_tilt: Angle | None = None,
        station_altitude: Distance | None = None,
    ) -> Decibel:
        """Compute rain attenuation exceeded for a given probability (P.618-13)."""
        ...
    def cloud_attenuation(
        self,
        lat: Angle,
        lon: Angle,
        elevation: Angle,
        frequency: Frequency,
        probability: float,
    ) -> Decibel:
        """Compute cloud attenuation on a slant path (P.840-9)."""
        ...
    def scintillation_attenuation(
        self,
        frequency: Frequency,
        elevation: Angle,
        probability: float,
        diameter: Distance,
        eta: float = 0.5,
        lat: Angle | None = None,
        lon: Angle | None = None,
    ) -> Decibel:
        """Compute tropospheric scintillation fade depth (P.618-13)."""
        ...
    def atmospheric_attenuation_slant_path(
        self,
        lat: Angle,
        lon: Angle,
        frequency: Frequency,
        elevation: Angle,
        probability: float,
        diameter: Distance,
        polarisation_tilt: Angle | None = None,
    ) -> EnvironmentalLosses:
        """Compute atmospheric attenuation on a slant path.

        Combines rain (P.618), gaseous (P.676), cloud (P.840), and
        scintillation (P.618) attenuation per ITU-R recommendations.
        """
        ...

def gaseous_attenuation_slant_path(
    frequency: Frequency,
    elevation: Angle,
    pressure: Pressure,
    rho: float,
    temperature: Temperature,
) -> tuple[Decibel, Decibel]:
    """Compute gaseous attenuation on a slant path (P.676-12).

    Returns:
        Tuple of (oxygen attenuation, water vapour attenuation).
    """
    ...

def rain_specific_attenuation(
    rain_rate: float,
    frequency: Frequency,
    elevation: Angle,
    polarisation_tilt: Angle | None = None,
) -> float:
    """Compute specific attenuation from rain in dB/km (P.838-3)."""
    ...
