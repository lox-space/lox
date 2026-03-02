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

class DataRate:
    """Data rate type for type-safe data rate values.

    Use with unit constants: `1e6 * lox.bps` or `10 * lox.Mbps`
    Convert to float with `float(rate)` (returns bits/s).
    """
    def __new__(cls, value: float) -> Self: ...
    def __add__(self, other: DataRate) -> DataRate: ...
    def __sub__(self, other: DataRate) -> DataRate: ...
    def __neg__(self) -> DataRate: ...
    def __mul__(self, other: float) -> DataRate: ...
    def __rmul__(self, other: float) -> DataRate: ...
    def __eq__(self, other: object) -> bool: ...
    def __repr__(self) -> str: ...
    def __str__(self) -> str: ...
    def __complex__(self) -> complex: ...
    def __float__(self) -> float: ...
    def __int__(self) -> int: ...
    def to_bits_per_second(self) -> float:
        """Returns the value in bits per second."""
        ...
    def to_kilobits_per_second(self) -> float:
        """Returns the value in kilobits per second."""
        ...
    def to_megabits_per_second(self) -> float:
        """Returns the value in megabits per second."""
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
bps: DataRate
"""1 bit per second"""
kbps: DataRate
"""1000 bits per second"""
Mbps: DataRate
"""1000000 bits per second"""
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

class GroundAsset:
    """A named ground station for visibility analysis.

    Wraps a ground location and elevation mask with an identifier.

    Args:
        id: Unique identifier for this ground asset.
        location: Ground station location.
        mask: Elevation mask defining minimum elevation constraints.

    Examples:
        >>> gs = lox.GroundAsset("ESOC", ground_location, elevation_mask)
    """
    def __new__(
        cls, id: str, location: GroundLocation, mask: ElevationMask
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

class SpaceAsset:
    """A named spacecraft for visibility analysis.

    Wraps a trajectory with an identifier.

    Args:
        id: Unique identifier for this space asset.
        trajectory: Spacecraft trajectory.

    Examples:
        >>> sc = lox.SpaceAsset("ISS", trajectory)
    """
    def __new__(cls, id: str, trajectory: Trajectory) -> Self: ...
    def id(self) -> str:
        """Return the asset identifier."""
        ...
    def trajectory(self) -> Trajectory:
        """Return the spacecraft trajectory."""
        ...

class VisibilityAnalysis:
    """Computes ground-station-to-spacecraft and inter-satellite visibility.

    Ground-to-space pairs are always computed when ground assets are present.
    Inter-satellite pairs are additionally computed when ``inter_satellite``
    is set to True.

    Args:
        ground_assets: List of GroundAsset objects.
        space_assets: List of SpaceAsset objects.
        occulting_bodies: Optional list of bodies for line-of-sight checking.
        step: Optional time step for event detection (default: 60s).
        min_pass_duration: Optional minimum pass duration. Passes
            shorter than this value may be missed. Enables two-level stepping
            for faster detection.
        inter_satellite: If True, also compute inter-satellite visibility
            for all unique spacecraft pairs (default: False).

    Examples:
        >>> analysis = lox.VisibilityAnalysis(
        ...     [ground_asset],
        ...     [space_asset],
        ...     step=lox.TimeDelta(60),
        ... )
        >>> results = analysis.compute(start, end, spk)
    """
    def __new__(
        cls,
        ground_assets: list[GroundAsset],
        space_assets: list[SpaceAsset],
        occulting_bodies: list[str | int | Origin] | None = None,
        step: TimeDelta | None = None,
        min_pass_duration: TimeDelta | None = None,
        inter_satellite: bool = False,
    ) -> Self: ...
    def compute(self, start: Time, end: Time, ephemeris: SPK) -> VisibilityResults:
        """Compute visibility intervals for all (ground, space) pairs.

        Args:
            start: Start time of the analysis period.
            end: End time of the analysis period.
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
    def orbital_period(self) -> TimeDelta:
        """Return the orbital period."""
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

class J2:
    """Numerical J2 orbit propagator using Dormand-Prince 8(5,3) integration.

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

class SGP4:
    """SGP4 (Simplified General Perturbations 4) orbit propagator.

    SGP4 is the standard propagator for objects tracked by NORAD/Space-Track.
    It uses Two-Line Element (TLE) data.

    Args:
        tle: Two-Line Element set (2 or 3 lines).

    Examples:
        >>> tle = '''ISS (ZARYA)
        ... 1 25544U 98067A   24001.50000000  .00016717  00000-0  10270-3 0  9002
        ... 2 25544  51.6400 208.9163 0006703  40.7490  46.4328 15.49952307    11'''
        >>> sgp4 = lox.SGP4(tle)
        >>> trajectory = sgp4.propagate([t1, t2, t3])
    """
    def __new__(cls, tle: str) -> Self: ...
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
        method: Interpolation method ("linear" or "cubic_spline").

    Examples:
        >>> x = [0.0, 1.0, 2.0, 3.0]
        >>> y = [0.0, 1.0, 4.0, 9.0]
        >>> series = lox.Series(x, y, method="cubic_spline")
        >>> series.interpolate(1.5)
        2.25
    """
    def __new__(
        cls,
        x: list[float],
        y: list[float],
        interpolation: Literal["linear", "cubic_spline"] = "linear",
    ) -> Self: ...
    def interpolate(self, xp: float) -> float:
        """Interpolate a y value at the given x coordinate."""
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
    def gain(self, frequency: Frequency, angle: Angle) -> Decibel:
        """Returns the gain in dBi at the given frequency and off-boresight angle."""
        ...
    def beamwidth(self, frequency: Frequency) -> Angle | None:
        """Returns the half-power beamwidth, or ``None`` when the
        antenna diameter is smaller than ~1.22 wavelengths at this frequency."""
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
    def gain(self, frequency: Frequency, angle: Angle) -> Decibel:
        """Returns the gain in dBi at the given frequency and off-boresight angle."""
        ...
    def beamwidth(self, frequency: Frequency) -> Angle | None:
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
    def gain(self, frequency: Frequency, angle: Angle) -> Decibel:
        """Returns the gain in dBi at the given frequency and off-boresight angle."""
        ...
    def peak_gain(self, frequency: Frequency) -> Decibel:
        """Returns the peak gain in dBi."""
        ...
    def __eq__(self, other: object) -> bool: ...
    def __repr__(self) -> str: ...

class SimpleAntenna:
    """A simple antenna with constant gain and beamwidth.

    Args:
        gain: Peak gain as Decibel.
        beamwidth: Half-power beamwidth as Angle.
    """
    def __new__(cls, gain: Decibel, beamwidth: Angle) -> Self: ...
    def __eq__(self, other: object) -> bool: ...
    def __repr__(self) -> str: ...

class ComplexAntenna:
    """An antenna with a physics-based gain pattern and boresight vector.

    Args:
        pattern: An antenna pattern (ParabolicPattern, GaussianPattern, or DipolePattern).
        boresight: Boresight direction as [x, y, z].
    """
    def __new__(
        cls,
        pattern: ParabolicPattern | GaussianPattern | DipolePattern,
        boresight: list[float],
    ) -> Self: ...
    def gain(self, frequency: Frequency, angle: Angle) -> Decibel:
        """Returns the gain in dBi at the given frequency and off-boresight angle."""
        ...
    def beamwidth(self, frequency: Frequency) -> Angle | None:
        """Returns the half-power beamwidth, or ``None`` when the
        underlying pattern does not define a beamwidth."""
        ...
    def peak_gain(self, frequency: Frequency) -> Decibel:
        """Returns the peak gain in dBi."""
        ...
    def __repr__(self) -> str: ...

class Transmitter:
    """A radio transmitter.

    Args:
        frequency: Transmit frequency.
        power: Transmit power.
        line_loss: Feed/line loss as Decibel.
        output_back_off: Output back-off as Decibel (default Decibel(0)).
    """
    def __new__(
        cls,
        frequency: Frequency,
        power: Power,
        line_loss: Decibel,
        output_back_off: Decibel | None = None,
    ) -> Self: ...
    def eirp(self, antenna: SimpleAntenna | ComplexAntenna, angle: Angle) -> Decibel:
        """Returns the EIRP in dBW for the given antenna and off-boresight angle."""
        ...
    def __eq__(self, other: object) -> bool: ...
    def __repr__(self) -> str: ...

class SimpleReceiver:
    """A simple receiver with a known system noise temperature.

    Args:
        frequency: Receive frequency.
        system_noise_temperature: System noise temperature.
    """
    def __new__(
        cls, frequency: Frequency, system_noise_temperature: Temperature
    ) -> Self: ...
    def __eq__(self, other: object) -> bool: ...
    def __repr__(self) -> str: ...

class ComplexReceiver:
    """A complex receiver with detailed noise and gain parameters.

    Args:
        frequency: Receive frequency.
        antenna_noise_temperature: Antenna noise temperature.
        lna_gain: LNA gain as Decibel.
        lna_noise_figure: LNA noise figure as Decibel.
        noise_figure: Receiver noise figure as Decibel.
        loss: Receiver chain loss as Decibel.
        demodulator_loss: Demodulator loss as Decibel (default Decibel(0)).
        implementation_loss: Other implementation losses as Decibel (default Decibel(0)).
    """
    def __new__(
        cls,
        frequency: Frequency,
        antenna_noise_temperature: Temperature,
        lna_gain: Decibel,
        lna_noise_figure: Decibel,
        noise_figure: Decibel,
        loss: Decibel,
        demodulator_loss: Decibel | None = None,
        implementation_loss: Decibel | None = None,
    ) -> Self: ...
    def noise_temperature(self) -> Temperature:
        """Returns the receiver noise temperature."""
        ...
    def system_noise_temperature(self) -> Temperature:
        """Returns the system noise temperature."""
        ...
    def __eq__(self, other: object) -> bool: ...
    def __repr__(self) -> str: ...

class Channel:
    """A communication channel.

    Args:
        link_type: "uplink" or "downlink".
        data_rate: Data rate.
        required_eb_n0: Required Eb/N0 as Decibel.
        margin: Required link margin as Decibel.
        modulation: Modulation scheme.
        roll_off: Roll-off factor (default 1.5).
        fec: Forward error correction code rate (default 0.5).
    """
    def __new__(
        cls,
        link_type: str,
        data_rate: DataRate,
        required_eb_n0: Decibel,
        margin: Decibel,
        modulation: Modulation,
        roll_off: float = 1.5,
        fec: float = 0.5,
    ) -> Self: ...
    def bandwidth(self) -> Frequency:
        """Returns the channel bandwidth."""
        ...
    def eb_n0(self, c_n0: Decibel) -> Decibel:
        """Computes Eb/N0 from a given C/N0."""
        ...
    def link_margin(self, eb_n0: Decibel) -> Decibel:
        """Computes the link margin from a given Eb/N0."""
        ...
    def __repr__(self) -> str: ...

class EnvironmentalLosses:
    """Environmental losses for a link.

    Args:
        rain: Rain attenuation as Decibel (default Decibel(0)).
        gaseous: Gaseous absorption as Decibel (default Decibel(0)).
        scintillation: Scintillation loss as Decibel (default Decibel(0)).
        atmospheric: Atmospheric loss as Decibel (default Decibel(0)).
        cloud: Cloud attenuation as Decibel (default Decibel(0)).
        depolarization: Depolarization loss as Decibel (default Decibel(0)).
    """
    def __new__(
        cls,
        rain: Decibel | None = None,
        gaseous: Decibel | None = None,
        scintillation: Decibel | None = None,
        atmospheric: Decibel | None = None,
        cloud: Decibel | None = None,
        depolarization: Decibel | None = None,
    ) -> Self: ...
    def total(self) -> Decibel:
        """Returns the total environmental loss in dB."""
        ...
    def __eq__(self, other: object) -> bool: ...
    def __repr__(self) -> str: ...

class CommunicationSystem:
    """A communication system combining an antenna with optional transmitter and receiver.

    Args:
        antenna: A SimpleAntenna or ComplexAntenna.
        receiver: A SimpleReceiver or ComplexReceiver (optional).
        transmitter: A Transmitter (optional).
    """
    def __new__(
        cls,
        antenna: SimpleAntenna | ComplexAntenna,
        receiver: SimpleReceiver | ComplexReceiver | None = None,
        transmitter: Transmitter | None = None,
    ) -> Self: ...
    def carrier_to_noise_density(
        self,
        rx_system: CommunicationSystem,
        losses: Decibel,
        range: Distance,
        tx_angle: Angle,
        rx_angle: Angle,
    ) -> Decibel:
        """Computes the carrier-to-noise density ratio (C/N0) in dB·Hz.

        Args:
            rx_system: The receiving CommunicationSystem.
            losses: Additional losses as Decibel.
            range: Slant range as Distance.
            tx_angle: Off-boresight angle at transmitter as Angle.
            rx_angle: Off-boresight angle at receiver as Angle.
        """
        ...
    def carrier_power(
        self,
        rx_system: CommunicationSystem,
        losses: Decibel,
        range: Distance,
        tx_angle: Angle,
        rx_angle: Angle,
    ) -> Decibel:
        """Computes the received carrier power in dBW."""
        ...
    def noise_power(self, bandwidth: Frequency) -> Decibel:
        """Computes the noise power in dBW for a given bandwidth."""
        ...
    def __repr__(self) -> str: ...

class LinkStats:
    """Complete link budget statistics."""
    @staticmethod
    def calculate(
        tx_system: CommunicationSystem,
        rx_system: CommunicationSystem,
        channel: Channel,
        range: Distance,
        tx_angle: Angle,
        rx_angle: Angle,
        losses: EnvironmentalLosses | None = None,
    ) -> LinkStats:
        """Computes a full link budget.

        Args:
            tx_system: The transmitting CommunicationSystem.
            rx_system: The receiving CommunicationSystem.
            channel: The Channel.
            range: Slant range as Distance.
            tx_angle: Off-boresight angle at transmitter as Angle.
            rx_angle: Off-boresight angle at receiver as Angle.
            losses: EnvironmentalLosses (optional, defaults to none).
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
    def eb_n0(self) -> Decibel:
        """Eb/N0 in dB."""
        ...
    @property
    def margin(self) -> Decibel:
        """Link margin in dB."""
        ...
    @property
    def carrier_rx_power(self) -> Decibel:
        """Received carrier power in dBW."""
        ...
    @property
    def noise_power(self) -> Decibel:
        """Noise power in dBW."""
        ...
    @property
    def data_rate(self) -> DataRate:
        """Data rate."""
        ...
    @property
    def bandwidth(self) -> Frequency:
        """Channel bandwidth."""
        ...
    @property
    def frequency(self) -> Frequency:
        """Link frequency."""
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
