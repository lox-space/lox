# SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
#
# SPDX-License-Identifier: MPL-2.0

import lox_space as lox
import numpy as np
import numpy.testing as npt
import pytest
import spiceypy as spice

@pytest.fixture(scope="session")
def kernels(data_dir):
    lsk = data_dir / "spice" / "naif0012.tls"
    pck = data_dir / "spice" / "pck00011.tpc"
    spice.furnsh([str(lsk), str(pck)])


@pytest.mark.parametrize(
    "frame",
    [
        "IAU_SUN",
        "IAU_MERCURY",
        "IAU_VENUS",
        "IAU_EARTH",
        "IAU_MARS",
        "IAU_JUPITER",
        "IAU_SATURN",
        "IAU_URANUS",
        "IAU_NEPTUNE",
        "IAU_PLUTO",
        "IAU_MOON",
        "IAU_PHOBOS",
        "IAU_DEIMOS",
        "IAU_IO",
        "IAU_EUROPA",
        "IAU_GANYMEDE",
        "IAU_CALLISTO",
        "IAU_AMALTHEA",
        "IAU_THEBE",
        "IAU_ADRASTEA",
        "IAU_METIS",
        "IAU_MIMAS",
        "IAU_ENCELADUS",
        "IAU_TETHYS",
        "IAU_DIONE",
        "IAU_RHEA",
        "IAU_TITAN",
        "IAU_IAPETUS",
        "IAU_PHOEBE",
        "IAU_JANUS",
        "IAU_EPIMETHEUS",
        "IAU_HELENE",
        "IAU_TELESTO",
        "IAU_CALYPSO",
        "IAU_ATLAS",
        "IAU_PROMETHEUS",
        "IAU_PANDORA",
        "IAU_PAN",
        "IAU_ARIEL",
        "IAU_UMBRIEL",
        "IAU_TITANIA",
        "IAU_OBERON",
        "IAU_MIRANDA",
        "IAU_CORDELIA",
        "IAU_OPHELIA",
        "IAU_BIANCA",
        "IAU_CRESSIDA",
        "IAU_DESDEMONA",
        "IAU_JULIET",
        "IAU_PORTIA",
        "IAU_ROSALIND",
        "IAU_BELINDA",
        "IAU_PUCK",
        "IAU_TRITON",
        "IAU_NAIAD",
        "IAU_THALASSA",
        "IAU_DESPINA",
        "IAU_GALATEA",
        "IAU_LARISSA",
        "IAU_PROTEUS",
        "IAU_CHARON",
        "IAU_GASPRA",
        "IAU_IDA",
        "IAU_CERES",
        "IAU_PALLAS",
        "IAU_VESTA",
        "IAU_LUTETIA",
        "IAU_EROS",
        "IAU_DAVIDA",
        "IAU_STEINS",
        "IAU_ITOKAWA",
    ],
)
def test_iau_frames(frame, kernels):
    t = lox.Time("TDB", 2000, 1, 1)
    et = t.julian_date(epoch="j2000", unit="seconds")
    r0 = (6068.27927, -1692.84394, -2516.61918)
    v0 = (-0.660415582, 5.495938726, -5.303093233)
    s0 = lox.State(t, position=r0, velocity=v0)
    s1 = s0.to_frame(lox.Frame(frame))
    r1_act = s1.position()
    v1_act = s1.velocity()
    ms = spice.sxform("J2000", frame, et)
    s1_exp = ms @ np.array([*r0, *v0])
    r1_exp = s1_exp[0:3]
    v1_exp = s1_exp[3:]
    npt.assert_allclose(r1_act, r1_exp)
    npt.assert_allclose(v1_act, v1_exp, atol=1e-3)
