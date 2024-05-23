class Time:
    def __init__(
        self,
        scale: str,
        year: int,
        month: int,
        day: int,
        hour: int = 0,
        minute: int = 0,
        second: int = 0,
        milli: int = 0,
        micro: int = 0,
        nano: int = 0,
        pico: int = 0,
        femto: int = 0,
    ): ...

    def scale(self) -> str: ...

    def to_tai(self, provider: UT1Provider | None = None) -> Time: ...

    def to_tcb(self, provider: UT1Provider | None = None) -> Time: ...

    def to_tcg(self, provider: UT1Provider | None = None) -> Time: ...

    def to_tdb(self, provider: UT1Provider | None = None) -> Time: ...

    def to_tt(self, provider: UT1Provider | None = None) -> Time: ...

    def to_ut1(self, provider: UT1Provider | None = None) -> Time: ...

    def to_utc(self, provider: UT1Provider | None = None) -> UTC: ...


class UTC:
    def __new__(
        cls,
        year: int,
        month: int,
        day: int,
        hour: int = 0,
        minute: int = 0,
        seconds: float = 0.0,
    ): ...

    def year(self) -> int: ...

    def month(self) -> int: ...

    def day(self) -> int: ...

    def hour(self) -> int: ...

    def minute(self) -> int: ...

    def second(self) -> int: ...

    def millisecond(self) -> int: ...

    def microsecond(self) -> int: ...

    def nanosecond(self) -> int: ...

    def picosecond(self) -> int: ...

    def decimal_seconds(self) -> float: ...

    def to_tai(self) -> Time: ...

    def to_tcb(self) -> Time: ...

    def to_tcg(self) -> Time: ...

    def to_tdb(self) -> Time: ...

    def to_tt(self) -> Time: ...

    def to_ut1(self, provider: UT1Provider) -> Time: ...


class UT1Provider:
    def __new__(cls, path: str): ...


class Sun:
    def __new__(cls): ...

    def id(self) -> int: ...

    def name(self) -> str: ...

    def gravitational_parameter(self) -> float: ...

    def mean_radius(self) -> float: ...

    def polar_radius(self) -> float: ...

    def equatorial_radius(self) -> float: ...


class Barycenter:
    def __new__(cls, name: str): ...

    def id(self) -> int: ...

    def name(self) -> str: ...

    def gravitational_parameter(self) -> float: ...


class Planet:
    def __new__(cls, name: str): ...

    def id(self) -> int: ...

    def name(self) -> str: ...

    def gravitational_parameter(self) -> float: ...

    def mean_radius(self) -> float: ...

    def polar_radius(self) -> float: ...

    def equatorial_radius(self) -> float: ...


class Satellite:
    def __new__(cls, name: str): ...

    def id(self) -> int: ...

    def name(self) -> str: ...

    def gravitational_parameter(self) -> float: ...

    def mean_radius(self) -> float: ...

    def polar_radius(self) -> float: ...

    def subplanetary_radius(self) -> float: ...

    def along_orbit_radius(self) -> float: ...


class MinorBody:
    def __new__(cls, name: str): ...

    def id(self) -> int: ...

    def name(self) -> str: ...

    def gravitational_parameter(self) -> float: ...

    def mean_radius(self) -> float: ...

    def polar_radius(self) -> float: ...

    def subplanetary_radius(self) -> float: ...

    def along_orbit_radius(self) -> float: ...


class Cartesian:
    def __new__(
        cls,
        time: Time,
        body: Sun | Barycenter | Planet | Satellite | MinorBody,
        frame: str,
        x: float,
        y: float,
        z: float,
        vx: float,
        vy: float,
        vz: float,
    ): ...

    def time(self) -> Time: ...

    def reference_frame(self) -> str: ...

    def origin(self) -> Sun | Barycenter | Planet | Satellite | MinorBody: ...

    def position(self) -> tuple[float, float, float]: ...

    def velocity(self) -> tuple[float, float, float]: ...

    def to_keplerian(self) -> Keplerian: ...


class Keplerian:
    def __new__(
        cls,
        time: Time,
        body: Sun | Barycenter | Planet | Satellite | MinorBody,
        frame: str,
        semi_major_axis: float,
        eccentricity: float,
        inclination: float,
        ascending_node: float,
        periapsis_argument: float,
        true_anomaly: float,
    ): ...

    def time(self) -> Time: ...

    def reference_frame(self) -> str: ...

    def origin(self) -> Sun | Barycenter | Planet | Satellite | MinorBody: ...

    def semi_major_axis(self) -> float: ...

    def eccentricity(self) -> float: ...

    def inclination(self) -> float: ...

    def ascending_node(self) -> float: ...

    def periapsis_argument(self) -> float: ...

    def true_anomaly(self) -> float: ...

    def to_cartesian(self) -> Cartesian: ...
