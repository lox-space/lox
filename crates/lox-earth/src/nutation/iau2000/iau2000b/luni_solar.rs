/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::nutation::iau2000::LuniSolarCoefficients;

#[rustfmt::skip]
// @formatter:off (sometimes RustRover ignores the rustfmt skip)
pub(super) const COEFFICIENTS: [LuniSolarCoefficients; 77] = [
    LuniSolarCoefficients{l:  0.0,  lp:  0.0,   f:  0.0,  d:  0.0,  om: 1.0,    sin_psi: -172064161.0,  sin_psi_t: -174666.0,   cos_psi: 33386.0,   cos_eps: 92052331.0,    cos_eps_t: 9086.0,  sin_eps: 15377.0},
    LuniSolarCoefficients{l:  0.0,  lp:  0.0,   f:  2.0,  d: -2.0,  om: 2.0,    sin_psi: -13170906.0,   sin_psi_t: -1675.0,     cos_psi: -13696.0,  cos_eps: 5730336.0,     cos_eps_t: -3015.0, sin_eps: -4587.0},
    LuniSolarCoefficients{l:  0.0,  lp:  0.0,   f:  2.0,  d:  0.0,  om: 2.0,    sin_psi: -2276413.0,    sin_psi_t: -234.0,      cos_psi: 2796.0,    cos_eps: 978459.0,      cos_eps_t: -485.0,  sin_eps: 1374.0},
    LuniSolarCoefficients{l:  0.0,  lp:  0.0,   f:  0.0,  d:  0.0,  om: 2.0,    sin_psi:  2074554.0,    sin_psi_t:   207.0,     cos_psi:  -698.0,   cos_eps: -897492.0,     cos_eps_t:  470.0,  sin_eps: -291.0},
    LuniSolarCoefficients{l:  0.0,  lp:  1.0,   f:  0.0,  d:  0.0,  om: 0.0,    sin_psi:  1475877.0,    sin_psi_t: -3633.0,     cos_psi: 11817.0,   cos_eps:  73871.0,      cos_eps_t: -184.0,  sin_eps: -1924.0},
    LuniSolarCoefficients{l:  0.0,  lp:  1.0,   f:  2.0,  d: -2.0,  om: 2.0,    sin_psi: -516821.0,     sin_psi_t: 1226.0,      cos_psi: -524.0,    cos_eps: 224386.0,      cos_eps_t: -677.0,  sin_eps: -174.0},
    LuniSolarCoefficients{l:  1.0,  lp:  0.0,   f:  0.0,  d:  0.0,  om: 0.0,    sin_psi:  711159.0,     sin_psi_t:   73.0,      cos_psi: -872.0,    cos_eps:  -6750.0,      cos_eps_t:    0.0,  sin_eps:  358.0},
    LuniSolarCoefficients{l:  0.0,  lp:  0.0,   f:  2.0,  d:  0.0,  om: 1.0,    sin_psi: -387298.0,     sin_psi_t: -367.0,      cos_psi:  380.0,    cos_eps: 200728.0,      cos_eps_t:   18.0,  sin_eps:  318.0},
    LuniSolarCoefficients{l:  1.0,  lp:  0.0,   f:  2.0,  d:  0.0,  om: 2.0,    sin_psi: -301461.0,     sin_psi_t:  -36.0,      cos_psi:  816.0,    cos_eps: 129025.0,      cos_eps_t:  -63.0,  sin_eps:  367.0},
    LuniSolarCoefficients{l:  0.0,  lp: -1.0,   f:  2.0,  d: -2.0,  om: 2.0,    sin_psi:  215829.0,     sin_psi_t: -494.0,      cos_psi:  111.0,    cos_eps: -95929.0,      cos_eps_t:  299.0,  sin_eps:  132.0},
    LuniSolarCoefficients{l:  0.0,  lp:  0.0,   f:  2.0,  d: -2.0,  om: 1.0,    sin_psi:  128227.0,     sin_psi_t:  137.0,      cos_psi:  181.0,    cos_eps: -68982.0,      cos_eps_t:   -9.0,  sin_eps:   39.0},
    LuniSolarCoefficients{l: -1.0,  lp:  0.0,   f:  2.0,  d:  0.0,  om: 2.0,    sin_psi:  123457.0,     sin_psi_t:   11.0,      cos_psi:   19.0,    cos_eps: -53311.0,      cos_eps_t:   32.0,  sin_eps:   -4.0},
    LuniSolarCoefficients{l: -1.0,  lp:  0.0,   f:  0.0,  d:  2.0,  om: 0.0,    sin_psi:  156994.0,     sin_psi_t:   10.0,      cos_psi: -168.0,    cos_eps:  -1235.0,      cos_eps_t:    0.0,  sin_eps:   82.0},
    LuniSolarCoefficients{l:  1.0,  lp:  0.0,   f:  0.0,  d:  0.0,  om: 1.0,    sin_psi:   63110.0,     sin_psi_t:   63.0,      cos_psi:   27.0,    cos_eps: -33228.0,      cos_eps_t:    0.0,  sin_eps:   -9.0},
    LuniSolarCoefficients{l: -1.0,  lp:  0.0,   f:  0.0,  d:  0.0,  om: 1.0,    sin_psi:  -57976.0,     sin_psi_t:  -63.0,      cos_psi: -189.0,    cos_eps:  31429.0,      cos_eps_t:    0.0,  sin_eps:  -75.0},
    LuniSolarCoefficients{l: -1.0,  lp:  0.0,   f:  2.0,  d:  2.0,  om: 2.0,    sin_psi:  -59641.0,     sin_psi_t:  -11.0,      cos_psi:  149.0,    cos_eps:  25543.0,      cos_eps_t:  -11.0,  sin_eps:   66.0},
    LuniSolarCoefficients{l:  1.0,  lp:  0.0,   f:  2.0,  d:  0.0,  om: 1.0,    sin_psi:  -51613.0,     sin_psi_t:  -42.0,      cos_psi:  129.0,    cos_eps:  26366.0,      cos_eps_t:    0.0,  sin_eps:   78.0},
    LuniSolarCoefficients{l: -2.0,  lp:  0.0,   f:  2.0,  d:  0.0,  om: 1.0,    sin_psi:   45893.0,     sin_psi_t:   50.0,      cos_psi:   31.0,    cos_eps: -24236.0,      cos_eps_t:  -10.0,  sin_eps:   20.0},
    LuniSolarCoefficients{l:  0.0,  lp:  0.0,   f:  0.0,  d:  2.0,  om: 0.0,    sin_psi:   63384.0,     sin_psi_t:   11.0,      cos_psi: -150.0,    cos_eps:  -1220.0,      cos_eps_t:    0.0,  sin_eps:   29.0},
    LuniSolarCoefficients{l:  0.0,  lp:  0.0,   f:  2.0,  d:  2.0,  om: 2.0,    sin_psi:  -38571.0,     sin_psi_t:   -1.0,      cos_psi:  158.0,    cos_eps:  16452.0,      cos_eps_t:  -11.0,  sin_eps:   68.0},
    LuniSolarCoefficients{l:  0.0,  lp: -2.0,   f:  2.0,  d: -2.0,  om: 2.0,    sin_psi:   32481.0,     sin_psi_t:    0.0,      cos_psi:    0.0,    cos_eps: -13870.0,      cos_eps_t:    0.0,  sin_eps:    0.0},
    LuniSolarCoefficients{l: -2.0,  lp:  0.0,   f:  0.0,  d:  2.0,  om: 0.0,    sin_psi:  -47722.0,     sin_psi_t:    0.0,      cos_psi:  -18.0,    cos_eps:    477.0,      cos_eps_t:    0.0,  sin_eps:  -25.0},
    LuniSolarCoefficients{l:  2.0,  lp:  0.0,   f:  2.0,  d:  0.0,  om: 2.0,    sin_psi:  -31046.0,     sin_psi_t:   -1.0,      cos_psi:  131.0,    cos_eps:  13238.0,      cos_eps_t:  -11.0,  sin_eps:   59.0},
    LuniSolarCoefficients{l:  1.0,  lp:  0.0,   f:  2.0,  d: -2.0,  om: 2.0,    sin_psi:   28593.0,     sin_psi_t:    0.0,      cos_psi:   -1.0,    cos_eps: -12338.0,      cos_eps_t:   10.0,  sin_eps:   -3.0},
    LuniSolarCoefficients{l: -1.0,  lp:  0.0,   f:  2.0,  d:  0.0,  om: 1.0,    sin_psi:   20441.0,     sin_psi_t:   21.0,      cos_psi:   10.0,    cos_eps: -10758.0,      cos_eps_t:    0.0,  sin_eps:   -3.0},
    LuniSolarCoefficients{l:  2.0,  lp:  0.0,   f:  0.0,  d:  0.0,  om: 0.0,    sin_psi:   29243.0,     sin_psi_t:    0.0,      cos_psi:  -74.0,    cos_eps:   -609.0,      cos_eps_t:    0.0,  sin_eps:   13.0},
    LuniSolarCoefficients{l:  0.0,  lp:  0.0,   f:  2.0,  d:  0.0,  om: 0.0,    sin_psi:   25887.0,     sin_psi_t:    0.0,      cos_psi:  -66.0,    cos_eps:   -550.0,      cos_eps_t:    0.0,  sin_eps:   11.0},
    LuniSolarCoefficients{l:  0.0,  lp:  1.0,   f:  0.0,  d:  0.0,  om: 1.0,    sin_psi:  -14053.0,     sin_psi_t:  -25.0,      cos_psi:   79.0,    cos_eps:   8551.0,      cos_eps_t:   -2.0,  sin_eps:  -45.0},
    LuniSolarCoefficients{l: -1.0,  lp:  0.0,   f:  0.0,  d:  2.0,  om: 1.0,    sin_psi:   15164.0,     sin_psi_t:   10.0,      cos_psi:   11.0,    cos_eps:  -8001.0,      cos_eps_t:    0.0,  sin_eps:   -1.0},
    LuniSolarCoefficients{l:  0.0,  lp:  2.0,   f:  2.0,  d: -2.0,  om: 2.0,    sin_psi:  -15794.0,     sin_psi_t:   72.0,      cos_psi:  -16.0,    cos_eps:   6850.0,      cos_eps_t:  -42.0,  sin_eps:   -5.0},
    LuniSolarCoefficients{l:  0.0,  lp:  0.0,   f: -2.0,  d:  2.0,  om: 0.0,    sin_psi:   21783.0,     sin_psi_t:    0.0,      cos_psi:   13.0,    cos_eps:   -167.0,      cos_eps_t:    0.0,  sin_eps:   13.0},
    LuniSolarCoefficients{l:  1.0,  lp:  0.0,   f:  0.0,  d: -2.0,  om: 1.0,    sin_psi:  -12873.0,     sin_psi_t:  -10.0,      cos_psi:  -37.0,    cos_eps:   6953.0,      cos_eps_t:    0.0,  sin_eps:  -14.0},
    LuniSolarCoefficients{l:  0.0,  lp: -1.0,   f:  0.0,  d:  0.0,  om: 1.0,    sin_psi:  -12654.0,     sin_psi_t:   11.0,      cos_psi:   63.0,    cos_eps:   6415.0,      cos_eps_t:    0.0,  sin_eps:   26.0},
    LuniSolarCoefficients{l: -1.0,  lp:  0.0,   f:  2.0,  d:  2.0,  om: 1.0,    sin_psi:  -10204.0,     sin_psi_t:    0.0,      cos_psi:   25.0,    cos_eps:   5222.0,      cos_eps_t:    0.0,  sin_eps:   15.0},
    LuniSolarCoefficients{l:  0.0,  lp:  2.0,   f:  0.0,  d:  0.0,  om: 0.0,    sin_psi:   16707.0,     sin_psi_t:  -85.0,      cos_psi:  -10.0,    cos_eps:    168.0,      cos_eps_t:   -1.0,  sin_eps:   10.0},
    LuniSolarCoefficients{l:  1.0,  lp:  0.0,   f:  2.0,  d:  2.0,  om: 2.0,    sin_psi:   -7691.0,     sin_psi_t:    0.0,      cos_psi:   44.0,    cos_eps:   3268.0,      cos_eps_t:    0.0,  sin_eps:   19.0},
    LuniSolarCoefficients{l: -2.0,  lp:  0.0,   f:  2.0,  d:  0.0,  om: 0.0,    sin_psi:  -11024.0,     sin_psi_t:    0.0,      cos_psi:  -14.0,    cos_eps:    104.0,      cos_eps_t:    0.0,  sin_eps:    2.0},
    LuniSolarCoefficients{l:  0.0,  lp:  1.0,   f:  2.0,  d:  0.0,  om: 2.0,    sin_psi:    7566.0,     sin_psi_t:  -21.0,      cos_psi:  -11.0,    cos_eps:  -3250.0,      cos_eps_t:    0.0,  sin_eps:   -5.0},
    LuniSolarCoefficients{l:  0.0,  lp:  0.0,   f:  2.0,  d:  2.0,  om: 1.0,    sin_psi:   -6637.0,     sin_psi_t:  -11.0,      cos_psi:   25.0,    cos_eps:   3353.0,      cos_eps_t:    0.0,  sin_eps:   14.0},
    LuniSolarCoefficients{l:  0.0,  lp: -1.0,   f:  2.0,  d:  0.0,  om: 2.0,    sin_psi:   -7141.0,     sin_psi_t:   21.0,      cos_psi:    8.0,    cos_eps:   3070.0,      cos_eps_t:    0.0,  sin_eps:    4.0},
    LuniSolarCoefficients{l:  0.0,  lp:  0.0,   f:  0.0,  d:  2.0,  om: 1.0,    sin_psi:   -6302.0,     sin_psi_t:  -11.0,      cos_psi:    2.0,    cos_eps:   3272.0,      cos_eps_t:    0.0,  sin_eps:    4.0},
    LuniSolarCoefficients{l:  1.0,  lp:  0.0,   f:  2.0,  d: -2.0,  om: 1.0,    sin_psi:    5800.0,     sin_psi_t:   10.0,      cos_psi:    2.0,    cos_eps:  -3045.0,      cos_eps_t:    0.0,  sin_eps:   -1.0},
    LuniSolarCoefficients{l:  2.0,  lp:  0.0,   f:  2.0,  d: -2.0,  om: 2.0,    sin_psi:    6443.0,     sin_psi_t:    0.0,      cos_psi:   -7.0,    cos_eps:  -2768.0,      cos_eps_t:    0.0,  sin_eps:   -4.0},
    LuniSolarCoefficients{l: -2.0,  lp:  0.0,   f:  0.0,  d:  2.0,  om: 1.0,    sin_psi:   -5774.0,     sin_psi_t:  -11.0,      cos_psi:  -15.0,    cos_eps:   3041.0,      cos_eps_t:    0.0,  sin_eps:   -5.0},
    LuniSolarCoefficients{l:  2.0,  lp:  0.0,   f:  2.0,  d:  0.0,  om: 1.0,    sin_psi:   -5350.0,     sin_psi_t:    0.0,      cos_psi:   21.0,    cos_eps:   2695.0,      cos_eps_t:    0.0,  sin_eps:   12.0},
    LuniSolarCoefficients{l:  0.0,  lp: -1.0,   f:  2.0,  d: -2.0,  om: 1.0,    sin_psi:   -4752.0,     sin_psi_t:  -11.0,      cos_psi:   -3.0,    cos_eps:   2719.0,      cos_eps_t:    0.0,  sin_eps:   -3.0},
    LuniSolarCoefficients{l:  0.0,  lp:  0.0,   f:  0.0,  d: -2.0,  om: 1.0,    sin_psi:   -4940.0,     sin_psi_t:  -11.0,      cos_psi:  -21.0,    cos_eps:   2720.0,      cos_eps_t:    0.0,  sin_eps:   -9.0},
    LuniSolarCoefficients{l: -1.0,  lp: -1.0,   f:  0.0,  d:  2.0,  om: 0.0,    sin_psi:    7350.0,     sin_psi_t:    0.0,      cos_psi:   -8.0,    cos_eps:    -51.0,      cos_eps_t:    0.0,  sin_eps:    4.0},
    LuniSolarCoefficients{l:  2.0,  lp:  0.0,   f:  0.0,  d: -2.0,  om: 1.0,    sin_psi:    4065.0,     sin_psi_t:    0.0,      cos_psi:    6.0,    cos_eps:  -2206.0,      cos_eps_t:    0.0,  sin_eps:    1.0},
    LuniSolarCoefficients{l:  1.0,  lp:  0.0,   f:  0.0,  d:  2.0,  om: 0.0,    sin_psi:    6579.0,     sin_psi_t:    0.0,      cos_psi:  -24.0,    cos_eps:   -199.0,      cos_eps_t:    0.0,  sin_eps:    2.0},
    LuniSolarCoefficients{l:  0.0,  lp:  1.0,   f:  2.0,  d: -2.0,  om: 1.0,    sin_psi:    3579.0,     sin_psi_t:    0.0,      cos_psi:    5.0,    cos_eps:  -1900.0,      cos_eps_t:    0.0,  sin_eps:    1.0},
    LuniSolarCoefficients{l:  1.0,  lp: -1.0,   f:  0.0,  d:  0.0,  om: 0.0,    sin_psi:    4725.0,     sin_psi_t:    0.0,      cos_psi:   -6.0,    cos_eps:    -41.0,      cos_eps_t:    0.0,  sin_eps:    3.0},
    LuniSolarCoefficients{l: -2.0,  lp:  0.0,   f:  2.0,  d:  0.0,  om: 2.0,    sin_psi:   -3075.0,     sin_psi_t:    0.0,      cos_psi:   -2.0,    cos_eps:   1313.0,      cos_eps_t:    0.0,  sin_eps:   -1.0},
    LuniSolarCoefficients{l:  3.0,  lp:  0.0,   f:  2.0,  d:  0.0,  om: 2.0,    sin_psi:   -2904.0,     sin_psi_t:    0.0,      cos_psi:   15.0,    cos_eps:   1233.0,      cos_eps_t:    0.0,  sin_eps:    7.0},
    LuniSolarCoefficients{l:  0.0,  lp: -1.0,   f:  0.0,  d:  2.0,  om: 0.0,    sin_psi:    4348.0,     sin_psi_t:    0.0,      cos_psi:  -10.0,    cos_eps:    -81.0,      cos_eps_t:    0.0,  sin_eps:    2.0},
    LuniSolarCoefficients{l:  1.0,  lp: -1.0,   f:  2.0,  d:  0.0,  om: 2.0,    sin_psi:   -2878.0,     sin_psi_t:    0.0,      cos_psi:    8.0,    cos_eps:   1232.0,      cos_eps_t:    0.0,  sin_eps:    4.0},
    LuniSolarCoefficients{l:  0.0,  lp:  0.0,   f:  0.0,  d:  1.0,  om: 0.0,    sin_psi:   -4230.0,     sin_psi_t:    0.0,      cos_psi:    5.0,    cos_eps:    -20.0,      cos_eps_t:    0.0,  sin_eps:   -2.0},
    LuniSolarCoefficients{l: -1.0,  lp: -1.0,   f:  2.0,  d:  2.0,  om: 2.0,    sin_psi:   -2819.0,     sin_psi_t:    0.0,      cos_psi:    7.0,    cos_eps:   1207.0,      cos_eps_t:    0.0,  sin_eps:    3.0},
    LuniSolarCoefficients{l: -1.0,  lp:  0.0,   f:  2.0,  d:  0.0,  om: 0.0,    sin_psi:   -4056.0,     sin_psi_t:    0.0,      cos_psi:    5.0,    cos_eps:     40.0,      cos_eps_t:    0.0,  sin_eps:   -2.0},
    LuniSolarCoefficients{l:  0.0,  lp: -1.0,   f:  2.0,  d:  2.0,  om: 2.0,    sin_psi:   -2647.0,     sin_psi_t:    0.0,      cos_psi:   11.0,    cos_eps:   1129.0,      cos_eps_t:    0.0,  sin_eps:    5.0},
    LuniSolarCoefficients{l: -2.0,  lp:  0.0,   f:  0.0,  d:  0.0,  om: 1.0,    sin_psi:   -2294.0,     sin_psi_t:    0.0,      cos_psi:  -10.0,    cos_eps:   1266.0,      cos_eps_t:    0.0,  sin_eps:   -4.0},
    LuniSolarCoefficients{l:  1.0,  lp:  1.0,   f:  2.0,  d:  0.0,  om: 2.0,    sin_psi:    2481.0,     sin_psi_t:    0.0,      cos_psi:   -7.0,    cos_eps:  -1062.0,      cos_eps_t:    0.0,  sin_eps:   -3.0},
    LuniSolarCoefficients{l:  2.0,  lp:  0.0,   f:  0.0,  d:  0.0,  om: 1.0,    sin_psi:    2179.0,     sin_psi_t:    0.0,      cos_psi:   -2.0,    cos_eps:  -1129.0,      cos_eps_t:    0.0,  sin_eps:   -2.0},
    LuniSolarCoefficients{l: -1.0,  lp:  1.0,   f:  0.0,  d:  1.0,  om: 0.0,    sin_psi:    3276.0,     sin_psi_t:    0.0,      cos_psi:    1.0,    cos_eps:     -9.0,      cos_eps_t:    0.0,  sin_eps:    0.0},
    LuniSolarCoefficients{l:  1.0,  lp:  1.0,   f:  0.0,  d:  0.0,  om: 0.0,    sin_psi:   -3389.0,     sin_psi_t:    0.0,      cos_psi:    5.0,    cos_eps:     35.0,      cos_eps_t:    0.0,  sin_eps:   -2.0},
    LuniSolarCoefficients{l:  1.0,  lp:  0.0,   f:  2.0,  d:  0.0,  om: 0.0,    sin_psi:    3339.0,     sin_psi_t:    0.0,      cos_psi:  -13.0,    cos_eps:   -107.0,      cos_eps_t:    0.0,  sin_eps:    1.0},
    LuniSolarCoefficients{l: -1.0,  lp:  0.0,   f:  2.0,  d: -2.0,  om: 1.0,    sin_psi:   -1987.0,     sin_psi_t:    0.0,      cos_psi:   -6.0,    cos_eps:   1073.0,      cos_eps_t:    0.0,  sin_eps:   -2.0},
    LuniSolarCoefficients{l:  1.0,  lp:  0.0,   f:  0.0,  d:  0.0,  om: 2.0,    sin_psi:   -1981.0,     sin_psi_t:    0.0,      cos_psi:    0.0,    cos_eps:    854.0,      cos_eps_t:    0.0,  sin_eps:    0.0},
    LuniSolarCoefficients{l: -1.0,  lp:  0.0,   f:  0.0,  d:  1.0,  om: 0.0,    sin_psi:    4026.0,     sin_psi_t:    0.0,      cos_psi: -353.0,    cos_eps:   -553.0,      cos_eps_t:    0.0,  sin_eps: -139.0},
    LuniSolarCoefficients{l:  0.0,  lp:  0.0,   f:  2.0,  d:  1.0,  om: 2.0,    sin_psi:    1660.0,     sin_psi_t:    0.0,      cos_psi:   -5.0,    cos_eps:   -710.0,      cos_eps_t:    0.0,  sin_eps:   -2.0},
    LuniSolarCoefficients{l: -1.0,  lp:  0.0,   f:  2.0,  d:  4.0,  om: 2.0,    sin_psi:   -1521.0,     sin_psi_t:    0.0,      cos_psi:    9.0,    cos_eps:    647.0,      cos_eps_t:    0.0,  sin_eps:    4.0},
    LuniSolarCoefficients{l: -1.0,  lp:  1.0,   f:  0.0,  d:  1.0,  om: 1.0,    sin_psi:    1314.0,     sin_psi_t:    0.0,      cos_psi:    0.0,    cos_eps:   -700.0,      cos_eps_t:    0.0,  sin_eps:    0.0},
    LuniSolarCoefficients{l:  0.0,  lp: -2.0,   f:  2.0,  d: -2.0,  om: 1.0,    sin_psi:   -1283.0,     sin_psi_t:    0.0,      cos_psi:    0.0,    cos_eps:    672.0,      cos_eps_t:    0.0,  sin_eps:    0.0},
    LuniSolarCoefficients{l:  1.0,  lp:  0.0,   f:  2.0,  d:  2.0,  om: 1.0,    sin_psi:   -1331.0,     sin_psi_t:    0.0,      cos_psi:    8.0,    cos_eps:    663.0,      cos_eps_t:    0.0,  sin_eps:    4.0},
    LuniSolarCoefficients{l: -2.0,  lp:  0.0,   f:  2.0,  d:  2.0,  om: 2.0,    sin_psi:    1383.0,     sin_psi_t:    0.0,      cos_psi:   -2.0,    cos_eps:   -594.0,      cos_eps_t:    0.0,  sin_eps:   -2.0},
    LuniSolarCoefficients{l: -1.0,  lp:  0.0,   f:  0.0,  d:  0.0,  om: 2.0,    sin_psi:    1405.0,     sin_psi_t:    0.0,      cos_psi:    4.0,    cos_eps:   -610.0,      cos_eps_t:    0.0,  sin_eps:    2.0},
    LuniSolarCoefficients{l:  1.0,  lp:  1.0,   f:  2.0,  d: -2.0,  om: 2.0,    sin_psi:    1290.0,     sin_psi_t:    0.0,      cos_psi:    0.0,    cos_eps:   -556.0,      cos_eps_t:    0.0,  sin_eps:    0.0}
];
