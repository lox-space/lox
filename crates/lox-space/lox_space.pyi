# SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
#
# SPDX-License-Identifier: MPL-2.0

from collections.abc import Callable
from typing import Literal, Self, overload
import numpy as np
import os

type Scale = Literal["TAI", "TCB", "TCG", "TDB", "TT", "UT1"]
type Epoch = Literal["jd", "mjd", "j1950", "j2000"]
type Unit = Literal["seconds", "days", "centuries"]
type Vec3 = tuple[float, float, float]

# Exceptions
class NonFiniteTimeDeltaError(Exception):
    """Raised when a TimeDelta operation produces a non-finite result."""
    ...

# Unit classes
class Angle:
    """Angle type for type-safe angular values.

    Use with unit constants: `45 * lox.deg` or `1.5 * lox.rad`
    Convert to float with `float(angle)`.
    """
    def __new__(cls, value: float) -> Self: ...
    def __rmul__(self, other: float) -> Self: ...
    def __repr__(self) -> str: ...
    def __str__(self) -> str: ...
    def __complex__(self) -> complex: ...
    def __float__(self) -> float: ...
    def __int__(self) -> int: ...

class Distance:
    """Distance type for type-safe length values.

    Use with unit constants: `100 * lox.km` or `1.5 * lox.au`
    Convert to float with `float(distance)`.
    """
    def __new__(cls, value: float) -> Self: ...
    def __rmul__(self, other: float) -> Self: ...
    def __repr__(self) -> str: ...
    def __str__(self) -> str: ...
    def __complex__(self) -> complex: ...
    def __float__(self) -> float: ...
    def __int__(self) -> int: ...

class Frequency:
    """Frequency type for type-safe frequency values.

    Use with unit constants: `2.4 * lox.ghz` or `100 * lox.mhz`
    Convert to float with `float(frequency)`.
    """
    def __new__(cls, value: float) -> Self: ...
    def __rmul__(self, other: float) -> Self: ...
    def __repr__(self) -> str: ...
    def __str__(self) -> str: ...
    def __complex__(self) -> complex: ...
    def __float__(self) -> float: ...
    def __int__(self) -> int: ...

class Velocity:
    """Velocity type for type-safe speed values.

    Use with unit constants: `7.8 * lox.kms` or `100 * lox.ms`
    Convert to float with `float(velocity)`.
    """
    def __new__(cls, value: float) -> Self: ...
    def __rmul__(self, other: float) -> Self: ...
    def __repr__(self) -> str: ...
    def __str__(self) -> str: ...
    def __complex__(self) -> complex: ...
    def __float__(self) -> float: ...
    def __int__(self) -> int: ...

# Unit constants
rad: Angle
"""1 radian"""
deg: Angle
"""π/180 radians"""
m: Distance
"""1 meter"""
km: Distance
"""1000 meters"""
au: Distance
"""1 astronomical unit"""
hz: Frequency
"""1 Hz"""
khz: Frequency
"""1 kHz"""
mhz: Frequency
"""1 MHz"""
ghz: Frequency
"""1 GHz"""
thz: Frequency
"""1 THz"""
ms: Velocity
"""1 m/s"""
kms: Velocity
"""1 km/s"""

class Ensemble:
    """Collection of named trajectories for batch visibility analysis.

    Args:
        ensemble: Dictionary mapping spacecraft names to their trajectories.

    Examples:
        >>> ensemble = lox.Ensemble({
        ...     "SC1": trajectory1,
        ...     "SC2": trajectory2,
        ... })
        >>> results = lox.visibility_all(times, ground_stations, ensemble, spk)
    """
    def __new__(cls, ensemble: dict[str, Trajectory]) -> Self: ...

class ElevationMask:
    """Defines elevation constraints for visibility analysis.

    An elevation mask specifies the minimum elevation angle required for
    visibility at different azimuth angles. Can be either fixed (constant)
    or variable (azimuth-dependent).

    Args:
        azimuth: Array of azimuth angles in radians (for variable mask).
        elevation: Array of minimum elevations in radians (for variable mask).
        min_elevation: Fixed minimum elevation in radians.

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
        min_elevation: float | None = None,
    ) -> Self: ...
    @classmethod
    def variable(cls, azimuth: np.ndarray, elevation: np.ndarray) -> Self:
        """Create a variable elevation mask from azimuth-dependent data."""
        ...
    @classmethod
    def fixed(cls, min_elevation: float) -> Self:
        """Create a fixed elevation mask with constant minimum elevation."""
        ...
    def azimuth(self) -> list[float] | None:
        """Return the azimuth array (for variable masks only)."""
        ...
    def elevation(self) -> list[float] | None:
        """Return the elevation array (for variable masks only)."""
        ...
    def fixed_elevation(self) -> float | None:
        """Return the fixed elevation value (for fixed masks only)."""
        ...
    def min_elevation(self, azimuth: float) -> float:
        """Return the minimum elevation at the given azimuth."""
        ...

def find_events(
    func: Callable[[float], float], start: Time, times: list[float]
) -> list[Event]:
    """Find events where a function crosses zero.

    Args:
        func: Function that takes a float (seconds from start) and returns a float.
        start: Reference time (epoch).
        times: Array of time offsets in seconds from start.

    Returns:
        List of Event objects at the detected zero-crossings.
    """
    ...

def find_windows(
    func: Callable[[float], float], start: Time, end: Time, times: list[float]
) -> list[Window]:
    """Find time windows where a function is positive.

    Args:
        func: Function that takes a float (seconds from start) and returns a float.
        start: Start time of the analysis period.
        end: End time of the analysis period.
        times: Array of time offsets in seconds from start.

    Returns:
        List of Window objects for intervals where the function is positive.
    """
    ...

def visibility(
    times: list[Time],
    gs: GroundLocation,
    mask: ElevationMask,
    sc: Trajectory,
    ephemeris: SPK,
    bodies: list[Origin] | None = None,
) -> list[Pass]:
    """Compute visibility passes between a ground station and spacecraft.

    Args:
        times: List of Time objects defining the analysis period.
        gs: Ground station location.
        mask: Elevation mask defining minimum elevation constraints.
        sc: Spacecraft trajectory.
        ephemeris: SPK ephemeris data.
        bodies: Optional list of bodies for occultation checking.

    Returns:
        List of Pass objects containing visibility windows and observables.

    Raises:
        ValueError: If ground station and spacecraft have different origins.
    """
    ...

def visibility_all(
    times: list[Time],
    ground_stations: dict[str, tuple[GroundLocation, ElevationMask]],
    spacecraft: Ensemble,
    ephemeris: SPK,
    bodies: list[Origin] | None = None,
) -> dict[str, dict[str, list[Pass]]]:
    """Compute visibility for multiple spacecraft and ground stations.

    Args:
        times: List of Time objects defining the analysis period.
        ground_stations: Dictionary mapping station names to (location, mask) tuples.
        spacecraft: Ensemble of spacecraft trajectories.
        ephemeris: SPK ephemeris data.
        bodies: Optional list of bodies for occultation checking.

    Returns:
        Nested dictionary: {spacecraft_name: {station_name: [passes]}}.
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
    def gravitational_parameter(self) -> float:
        """Return the gravitational parameter (GM) in km³/s²."""
        ...
    def mean_radius(self) -> float:
        """Return the mean radius in km."""
        ...
    def radii(self) -> tuple[float, float, float]:
        """Return the triaxial radii (x, y, z) in km."""
        ...
    def equatorial_radius(self) -> float:
        """Return the equatorial radius in km."""
        ...
    def polar_radius(self) -> float:
        """Return the polar radius in km."""
        ...
    def rotational_elements(self, et: float) -> tuple[float, float, float]:
        """Return rotational elements (right ascension, declination, rotation angle) in radians."""
        ...
    def rotational_element_rates(self, et: float) -> tuple[float, float, float]:
        """Return rotational element rates in radians/second."""
        ...
    def right_ascension(self, et: float) -> float:
        """Return the right ascension of the pole in radians."""
        ...
    def right_ascension_rate(self, et: float) -> float:
        """Return the rate of change of right ascension in radians/second."""
        ...
    def declination(self, et: float) -> float:
        """Return the declination of the pole in radians."""
        ...
    def declination_rate(self, et: float) -> float:
        """Return the rate of change of declination in radians/second."""
        ...
    def rotation_angle(self, et: float) -> float:
        """Return the rotation angle (prime meridian) in radians."""
        ...
    def rotation_rate(self, et: float) -> float:
        """Return the rotation rate in radians/second."""
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

class State:
    """Represents an orbital state (position and velocity) at a specific time.

    Args:
        time: The epoch of this state.
        position: Position vector (x, y, z) in km.
        velocity: Velocity vector (vx, vy, vz) in km/s.
        origin: Central body (default: Earth).
        frame: Reference frame (default: ICRF).

    Examples:
        >>> t = lox.Time("TAI", 2024, 1, 1)
        >>> state = lox.State(
        ...     t,
        ...     position=(6678.0, 0.0, 0.0),
        ...     velocity=(0.0, 7.73, 0.0),
        ... )
    """
    def __new__(
        cls,
        time: Time,
        position: Vec3,
        velocity: Vec3,
        origin: Origin | None = None,
        frame: Frame | None = None,
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
        """Return the position vector as a numpy array [x, y, z] in km."""
        ...
    def velocity(self) -> np.ndarray:
        """Return the velocity vector as a numpy array [vx, vy, vz] in km/s."""
        ...
    def to_frame(self, frame: Frame, provider: EOPProvider | None = None) -> Self:
        """Transform this state to a different reference frame."""
        ...
    def to_origin(self, target: Origin, ephemeris: SPK) -> Self:
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

class Keplerian:
    """Represents an orbit using Keplerian (classical) orbital elements.

    Args:
        time: Epoch of the elements.
        semi_major_axis: Semi-major axis in km.
        eccentricity: Orbital eccentricity (0 = circular, <1 = elliptical).
        inclination: Inclination in radians.
        longitude_of_ascending_node: RAAN in radians.
        argument_of_periapsis: Argument of periapsis in radians.
        true_anomaly: True anomaly in radians.
        origin: Central body (default: Earth).

    Examples:
        >>> import math
        >>> t = lox.Time("TAI", 2024, 1, 1)
        >>> orbit = lox.Keplerian(
        ...     t,
        ...     semi_major_axis=6678.0,
        ...     eccentricity=0.001,
        ...     inclination=math.radians(51.6),
        ...     longitude_of_ascending_node=0.0,
        ...     argument_of_periapsis=0.0,
        ...     true_anomaly=0.0,
        ... )
    """
    def __new__(
        cls,
        time: Time,
        semi_major_axis: float,
        eccentricity: float,
        inclination: float,
        longitude_of_ascending_node: float,
        argument_of_periapsis: float,
        true_anomaly: float,
        origin: Origin | None = None,
    ) -> Self: ...
    def time(self) -> Time:
        """Return the epoch of these elements."""
        ...
    def origin(self) -> Origin:
        """Return the central body (origin) of this orbit."""
        ...
    def semi_major_axis(self) -> float:
        """Return the semi-major axis in km."""
        ...
    def eccentricity(self) -> float:
        """Return the orbital eccentricity."""
        ...
    def inclination(self) -> float:
        """Return the inclination in radians."""
        ...
    def longitude_of_ascending_node(self) -> float:
        """Return the longitude of the ascending node (RAAN) in radians."""
        ...
    def argument_of_periapsis(self) -> float:
        """Return the argument of periapsis in radians."""
        ...
    def true_anomaly(self) -> float:
        """Return the true anomaly in radians."""
        ...
    def to_cartesian(self) -> State:
        """Convert these Keplerian elements to a Cartesian state."""
        ...
    def orbital_period(self) -> TimeDelta:
        """Return the orbital period."""
        ...

class Trajectory:
    """A time-series of orbital states with interpolation support.

    Args:
        states: List of State objects in chronological order.

    Examples:
        >>> trajectory = propagator.propagate(times)
        >>> state = trajectory.interpolate(t)
        >>> arr = trajectory.to_numpy()
    """
    def __new__(cls, states: list[State]) -> Self: ...
    @classmethod
    def from_numpy(
        cls,
        start_time: Time,
        states: np.ndarray,
        origin: Origin | None = None,
        frame: Frame | None = None,
    ) -> Self:
        """Create a Trajectory from a numpy array.

        Args:
            start_time: Reference epoch for the trajectory.
            states: 2D array with columns [t, x, y, z, vx, vy, vz].
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
        """Export trajectory to a numpy array."""
        ...
    def states(self) -> list[State]:
        """Return the list of states in this trajectory."""
        ...
    def find_events(self, func: Callable[[State], float]) -> list[Event]:
        """Find events where a function crosses zero."""
        ...
    def find_windows(self, func: Callable[[State], float]) -> list[Window]:
        """Find time windows where a function is positive."""
        ...
    def interpolate(self, time: Time | TimeDelta) -> State:
        """Interpolate the trajectory at a specific time."""
        ...
    def to_frame(self, frame: Frame, provider: EOPProvider | None = None) -> Self:
        """Transform all states to a different reference frame."""
        ...
    def to_origin(self, target: Origin, ephemeris: SPK) -> Self:
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

class Window:
    """Represents a time window (interval between two times).

    Windows represent periods when certain conditions are met,
    such as visibility windows.
    """
    def __repr__(self) -> str: ...
    def start(self) -> Time:
        """Return the start time of this window."""
        ...
    def end(self) -> Time:
        """Return the end time of this window."""
        ...
    def duration(self) -> TimeDelta:
        """Return the duration of this window."""
        ...

class Vallado:
    """Semi-analytical Keplerian orbit propagator using Vallado's method.

    Args:
        initial_state: Initial orbital state (must be in an inertial frame).
        max_iter: Maximum iterations for Kepler's equation solver.

    Examples:
        >>> state = lox.State(t, position=(6678.0, 0.0, 0.0), velocity=(0.0, 7.73, 0.0))
        >>> prop = lox.Vallado(state)
        >>> trajectory = prop.propagate([t1, t2, t3])
    """
    def __new__(cls, initial_state: State, max_iter: int | None = None) -> Self: ...
    @overload
    def propagate(self, time: Time) -> State: ...
    @overload
    def propagate(self, time: list[Time]) -> Trajectory: ...
    def propagate(self, time: Time | list[Time]) -> State | Trajectory:
        """Propagate the orbit to one or more times."""
        ...

class GroundLocation:
    """Represents a location on the surface of a celestial body.

    Args:
        origin: The central body (e.g., Earth, Moon).
        longitude: Geodetic longitude in radians.
        latitude: Geodetic latitude in radians.
        altitude: Altitude above the reference ellipsoid in km.

    Examples:
        >>> import math
        >>> darmstadt = lox.GroundLocation(
        ...     lox.Origin("Earth"),
        ...     longitude=math.radians(8.6512),
        ...     latitude=math.radians(49.8728),
        ...     altitude=0.108,
        ... )
    """
    def __new__(
        cls,
        origin: Origin,
        longitude: float,
        latitude: float,
        altitude: float,
    ) -> Self: ...
    def longitude(self) -> float:
        """Return the geodetic longitude in radians."""
        ...
    def latitude(self) -> float:
        """Return the geodetic latitude in radians."""
        ...
    def altitude(self) -> float:
        """Return the altitude above the reference ellipsoid in km."""
        ...
    def observables(
        self,
        state: State,
        provider: EOPProvider | None = None,
        frame: Frame | None = None,
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
    def propagate(self, time: Time) -> State: ...
    @overload
    def propagate(self, time: list[Time]) -> Trajectory: ...
    def propagate(self, time: Time | list[Time]) -> State | Trajectory:
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
    def propagate(self, time: Time, provider: EOPProvider | None = None) -> State: ...
    @overload
    def propagate(self, time: list[Time], provider: EOPProvider | None = None) -> Trajectory: ...
    def propagate(self, time: Time | list[Time], provider: EOPProvider | None = None) -> State | Trajectory:
        """Propagate the orbit to one or more times."""
        ...

class Observables:
    """Observation data from a ground station to a target.

    Args:
        azimuth: Azimuth angle in radians (measured from north, clockwise).
        elevation: Elevation angle in radians (above local horizon).
        range: Distance to target in km.
        range_rate: Rate of change of range in km/s.
    """
    def __new__(
        cls, azimuth: float, elevation: float, range: float, range_rate: float
    ) -> Self: ...
    def azimuth(self) -> float:
        """Return the azimuth angle in radians."""
        ...
    def elevation(self) -> float:
        """Return the elevation angle in radians."""
        ...
    def range(self) -> float:
        """Return the range (distance) in km."""
        ...
    def range_rate(self) -> float:
        """Return the range rate in km/s."""
        ...

class Pass:
    """Represents a visibility pass between a ground station and spacecraft.

    A Pass contains the visibility window (start and end times) along with
    observables computed at regular intervals throughout the pass.
    """
    def __new__(
        cls, window: Window, times: list[Time], observables: list[Observables]
    ) -> Self: ...
    def window(self) -> Window:
        """Return the visibility window for this pass."""
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
        method: Literal["linear", "cubic_spline"] = "linear",
    ) -> Self: ...
    def interpolate(self, xp: float) -> float:
        """Interpolate a y value at the given x coordinate."""
        ...
