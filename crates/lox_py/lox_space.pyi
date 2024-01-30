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
            atto: int = 0,
    ): ...

    def days_since_j2000(self) -> float: ...

    def scale(self) -> str: ...


class Sun:
    def __init__(self): ...

    def id(self) -> int: ...

    def name(self) -> str: ...

    def gravitational_parameter(self) -> float: ...

    def mean_radius(self) -> float: ...

    def polar_radius(self) -> float: ...

    def equatorial_radius(self) -> float: ...


class Barycenter:
    def __init__(self, name: str): ...

    def id(self) -> int: ...

    def name(self) -> str: ...

    def gravitational_parameter(self) -> float: ...


class Planet:
    def __init__(self, name: str): ...

    def id(self) -> int: ...

    def name(self) -> str: ...

    def gravitational_parameter(self) -> float: ...

    def mean_radius(self) -> float: ...

    def polar_radius(self) -> float: ...

    def equatorial_radius(self) -> float: ...


class Satellite:
    def __init__(self, name: str): ...

    def id(self) -> int: ...

    def name(self) -> str: ...

    def gravitational_parameter(self) -> float: ...

    def mean_radius(self) -> float: ...

    def polar_radius(self) -> float: ...

    def subplanetary_radius(self) -> float: ...

    def along_orbit_radius(self) -> float: ...


class MinorBody:
    def __init__(self, name: str): ...

    def id(self) -> int: ...

    def name(self) -> str: ...

    def gravitational_parameter(self) -> float: ...

    def mean_radius(self) -> float: ...

    def polar_radius(self) -> float: ...

    def subplanetary_radius(self) -> float: ...

    def along_orbit_radius(self) -> float: ...


class Cartesian:
    def __init__(
            self,
            time: Epoch,
            body: Sun | Barycenter | Planet | Satellite | MinorBody,
            frame: str,
            x: float,
            y: float,
            z: float,
            vx: float,
            vy: float,
            vz: float,
    ): ...

    def time(self) -> Epoch: ...

    def reference_frame(self) -> str: ...

    def origin(self) -> Sun | Barycenter | Planet | Satellite | MinorBody: ...

    def position(self) -> tuple[float, float, float]: ...

    def velocity(self) -> tuple[float, float, float]: ...

    def to_keplerian(self) -> Keplerian: ...


class Keplerian:
    def __init__(
            self,
            time: Epoch,
            body: Sun | Barycenter | Planet | Satellite | MinorBody,
            frame: str,
            semi_major_axis: float,
            eccentricity: float,
            inclination: float,
            ascending_node: float,
            periapsis_argument: float,
            true_anomaly: float,
    ): ...

    def time(self) -> Epoch: ...

    def reference_frame(self) -> str: ...

    def origin(self) -> Sun | Barycenter | Planet | Satellite | MinorBody: ...

    def semi_major_axis(self) -> float: ...

    def eccentricity(self) -> float: ...

    def inclination(self) -> float: ...

    def ascending_node(self) -> float: ...

    def periapsis_argument(self) -> float: ...

    def true_anomaly(self) -> float: ...

    def to_cartesian(self) -> Cartesian: ...
