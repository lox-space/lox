#  Copyright (c) 2024. Helge Eichhorn and the LOX contributors
#
#  This Source Code Form is subject to the terms of the Mozilla Public
#  License, v. 2.0. If a copy of the MPL was not distributed with this
#  file, you can obtain one at https://mozilla.org/MPL/2.0/.

import lox_space as lox
import numpy as np

time = lox.Epoch("TDB", 2016, 5, 30, 12)
position = np.array([6068279.27, -1692843.94, -2516619.18]) * 1e-3
velocity = np.array([-660.415582, 5495.938726, -5303.093233]) * 1e-3
iss_cartesian = lox.Cartesian(time, lox.Planet("Earth"), "ICRF", position, velocity)
iss = iss_cartesian.to_keplerian()

print(f"Semi-major axis: {iss.semi_major_axis():.3f} km")
print(f"Eccentricity: {iss.eccentricity():.6f}")
print(f"Inclination: {np.degrees(iss.inclination()):.3f}°")
print(f"Longitude of ascending node: {np.degrees(iss.ascending_node()):.3f}°")
print(f"Argument of perigee: {np.degrees(iss.periapsis_argument()):.3f}°")
print(f"True anomaly: {np.degrees(iss.true_anomaly()):.3f}°")
