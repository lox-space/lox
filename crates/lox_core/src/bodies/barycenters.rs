// Auto-generated by `lox_gen`. Do not edit!
use super::{BarycenterTrigRotationalElements, NaifId, PointMass, PolynomialCoefficient};
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct SolarSystemBarycenter;
impl NaifId for SolarSystemBarycenter {
    fn id() -> i32 {
        0i32
    }
}
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct MercuryBarycenter;
impl NaifId for MercuryBarycenter {
    fn id() -> i32 {
        1i32
    }
}
impl PointMass for MercuryBarycenter {
    fn gravitational_parameter() -> f64 {
        22031.868551400003f64
    }
}
impl BarycenterTrigRotationalElements for MercuryBarycenter {
    const NUT_PREC_ANGLES: &'static [PolynomialCoefficient] = &[
        174.7910857f64,
        149472.53587500003f64,
        349.5821714f64,
        298945.07175000006f64,
        164.3732571f64,
        448417.60762500006f64,
        339.1643429f64,
        597890.1435000001f64,
        153.9554286f64,
        747362.679375f64,
    ];
}
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct VenusBarycenter;
impl NaifId for VenusBarycenter {
    fn id() -> i32 {
        2i32
    }
}
impl PointMass for VenusBarycenter {
    fn gravitational_parameter() -> f64 {
        324858.592f64
    }
}
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct EarthBarycenter;
impl NaifId for EarthBarycenter {
    fn id() -> i32 {
        3i32
    }
}
impl PointMass for EarthBarycenter {
    fn gravitational_parameter() -> f64 {
        403503.2356254802f64
    }
}
impl BarycenterTrigRotationalElements for EarthBarycenter {
    const NUT_PREC_ANGLES: &'static [PolynomialCoefficient] = &[
        125.045f64,
        -1935.5364525f64,
        250.089f64,
        -3871.072905f64,
        260.008f64,
        475263.3328725f64,
        176.625f64,
        487269.629985f64,
        357.529f64,
        35999.0509575f64,
        311.589f64,
        964468.49931f64,
        134.963f64,
        477198.869325f64,
        276.617f64,
        12006.300765f64,
        34.226f64,
        63863.5132425f64,
        15.134f64,
        -5806.6093575f64,
        119.743f64,
        131.84064f64,
        239.961f64,
        6003.1503825f64,
        25.053f64,
        473327.79642f64,
    ];
}
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct MarsBarycenter;
impl NaifId for MarsBarycenter {
    fn id() -> i32 {
        4i32
    }
}
impl PointMass for MarsBarycenter {
    fn gravitational_parameter() -> f64 {
        42828.3758157561f64
    }
}
impl BarycenterTrigRotationalElements for MarsBarycenter {
    const NUT_PREC_ANGLES: &'static [PolynomialCoefficient] = &[
        190.72646643f64,
        15917.10818695f64,
        0f64,
        21.4689247f64,
        31834.27934054f64,
        0f64,
        332.86082793f64,
        19139.89694742f64,
        0f64,
        394.93256437f64,
        38280.79631835f64,
        0f64,
        189.6327156f64,
        41215158.1842005f64,
        12.711923222f64,
        121.46893664f64,
        660.22803474f64,
        0f64,
        231.05028581f64,
        660.9912354f64,
        0f64,
        251.37314025f64,
        1320.50145245f64,
        0f64,
        217.98635955f64,
        38279.9612555f64,
        0f64,
        196.19729402f64,
        19139.83628608f64,
        0f64,
        198.991226f64,
        19139.4819985f64,
        0f64,
        226.292679f64,
        38280.8511281f64,
        0f64,
        249.663391f64,
        57420.7251593f64,
        0f64,
        266.18351f64,
        76560.636795f64,
        0f64,
        79.398797f64,
        0.5042615f64,
        0f64,
        122.433576f64,
        19139.9407476f64,
        0f64,
        43.058401f64,
        38280.8753272f64,
        0f64,
        57.663379f64,
        57420.7517205f64,
        0f64,
        79.476401f64,
        76560.6495004f64,
        0f64,
        166.325722f64,
        0.5042615f64,
        0f64,
        129.071773f64,
        19140.0328244f64,
        0f64,
        36.352167f64,
        38281.0473591f64,
        0f64,
        56.668646f64,
        57420.929536f64,
        0f64,
        67.364003f64,
        76560.2552215f64,
        0f64,
        104.79268f64,
        95700.4387578f64,
        0f64,
        95.391654f64,
        0.5042615f64,
        0f64,
    ];
}
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct JupiterBarycenter;
impl NaifId for JupiterBarycenter {
    fn id() -> i32 {
        5i32
    }
}
impl PointMass for JupiterBarycenter {
    fn gravitational_parameter() -> f64 {
        126712764.09999998f64
    }
}
impl BarycenterTrigRotationalElements for JupiterBarycenter {
    const NUT_PREC_ANGLES: &'static [PolynomialCoefficient] = &[
        73.32f64,
        91472.9f64,
        24.62f64,
        45137.2f64,
        283.9f64,
        4850.7f64,
        355.8f64,
        1191.3f64,
        119.9f64,
        262.1f64,
        229.8f64,
        64.3f64,
        352.25f64,
        2382.6f64,
        113.35f64,
        6070f64,
        146.64f64,
        182945.8f64,
        49.24f64,
        90274.4f64,
        99.360714f64,
        4850.4046f64,
        175.895369f64,
        1191.9605f64,
        300.323162f64,
        262.5475f64,
        114.012305f64,
        6070.2476f64,
        49.511251f64,
        64.3f64,
    ];
}
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct SaturnBarycenter;
impl NaifId for SaturnBarycenter {
    fn id() -> i32 {
        6i32
    }
}
impl PointMass for SaturnBarycenter {
    fn gravitational_parameter() -> f64 {
        37940584.8418f64
    }
}
impl BarycenterTrigRotationalElements for SaturnBarycenter {
    const NUT_PREC_ANGLES: &'static [PolynomialCoefficient] = &[
        353.32f64,
        75706.7f64,
        28.72f64,
        75706.7f64,
        177.4f64,
        -36505.5f64,
        300f64,
        -7225.9f64,
        316.45f64,
        506.2f64,
        345.2f64,
        -1016.3f64,
        706.64f64,
        151413.4f64,
        57.44f64,
        151413.4f64,
    ];
}
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct UranusBarycenter;
impl NaifId for UranusBarycenter {
    fn id() -> i32 {
        7i32
    }
}
impl PointMass for UranusBarycenter {
    fn gravitational_parameter() -> f64 {
        5794556.3999999985f64
    }
}
impl BarycenterTrigRotationalElements for UranusBarycenter {
    const NUT_PREC_ANGLES: &'static [PolynomialCoefficient] = &[
        115.75f64,
        54991.87f64,
        141.69f64,
        41887.66f64,
        135.03f64,
        29927.35f64,
        61.77f64,
        25733.59f64,
        249.32f64,
        24471.46f64,
        43.86f64,
        22278.41f64,
        77.66f64,
        20289.42f64,
        157.36f64,
        16652.76f64,
        101.81f64,
        12872.63f64,
        138.64f64,
        8061.81f64,
        102.23f64,
        -2024.22f64,
        316.41f64,
        2863.96f64,
        304.01f64,
        -51.94f64,
        308.71f64,
        -93.17f64,
        340.82f64,
        -75.32f64,
        259.14f64,
        -504.81f64,
        204.46f64,
        -4048.44f64,
        632.82f64,
        5727.92f64,
    ];
}
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct NeptuneBarycenter;
impl NaifId for NeptuneBarycenter {
    fn id() -> i32 {
        8i32
    }
}
impl PointMass for NeptuneBarycenter {
    fn gravitational_parameter() -> f64 {
        6836527.100580399f64
    }
}
impl BarycenterTrigRotationalElements for NeptuneBarycenter {
    const NUT_PREC_ANGLES: &'static [PolynomialCoefficient] = &[
        357.85f64,
        52.316f64,
        323.92f64,
        62606.6f64,
        220.51f64,
        55064.2f64,
        354.27f64,
        46564.5f64,
        75.31f64,
        26109.4f64,
        35.36f64,
        14325.4f64,
        142.61f64,
        2824.6f64,
        177.85f64,
        52.316f64,
        647.84f64,
        125213.2f64,
        355.7f64,
        104.632f64,
        533.55f64,
        156.948f64,
        711.4f64,
        209.264f64,
        889.25f64,
        261.58f64,
        1067.1f64,
        313.896f64,
        1244.95f64,
        366.212f64,
        1422.8f64,
        418.528f64,
        1600.65f64,
        470.844f64,
    ];
}
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct PlutoBarycenter;
impl NaifId for PlutoBarycenter {
    fn id() -> i32 {
        9i32
    }
}
impl PointMass for PlutoBarycenter {
    fn gravitational_parameter() -> f64 {
        975.5f64
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_naif_id_0() {
        assert_eq!(SolarSystemBarycenter::id(), 0i32)
    }
    #[test]
    fn test_naif_id_1() {
        assert_eq!(MercuryBarycenter::id(), 1i32)
    }
    #[test]
    fn test_point_mass_1() {
        assert_eq!(
            MercuryBarycenter::gravitational_parameter(),
            22031.868551400003f64
        );
    }
    #[test]
    fn test_barycenter_trig_rotational_elements_nut_prec_angles_1() {
        assert_eq!(
            &[
                174.7910857f64,
                149472.53587500003f64,
                349.5821714f64,
                298945.07175000006f64,
                164.3732571f64,
                448417.60762500006f64,
                339.1643429f64,
                597890.1435000001f64,
                153.9554286f64,
                747362.679375f64
            ],
            MercuryBarycenter::NUT_PREC_ANGLES
        )
    }
    #[test]
    fn test_naif_id_2() {
        assert_eq!(VenusBarycenter::id(), 2i32)
    }
    #[test]
    fn test_point_mass_2() {
        assert_eq!(VenusBarycenter::gravitational_parameter(), 324858.592f64);
    }
    #[test]
    fn test_naif_id_3() {
        assert_eq!(EarthBarycenter::id(), 3i32)
    }
    #[test]
    fn test_point_mass_3() {
        assert_eq!(
            EarthBarycenter::gravitational_parameter(),
            403503.2356254802f64
        );
    }
    #[test]
    fn test_barycenter_trig_rotational_elements_nut_prec_angles_3() {
        assert_eq!(
            &[
                125.045f64,
                -1935.5364525f64,
                250.089f64,
                -3871.072905f64,
                260.008f64,
                475263.3328725f64,
                176.625f64,
                487269.629985f64,
                357.529f64,
                35999.0509575f64,
                311.589f64,
                964468.49931f64,
                134.963f64,
                477198.869325f64,
                276.617f64,
                12006.300765f64,
                34.226f64,
                63863.5132425f64,
                15.134f64,
                -5806.6093575f64,
                119.743f64,
                131.84064f64,
                239.961f64,
                6003.1503825f64,
                25.053f64,
                473327.79642f64
            ],
            EarthBarycenter::NUT_PREC_ANGLES
        )
    }
    #[test]
    fn test_naif_id_4() {
        assert_eq!(MarsBarycenter::id(), 4i32)
    }
    #[test]
    fn test_point_mass_4() {
        assert_eq!(
            MarsBarycenter::gravitational_parameter(),
            42828.3758157561f64
        );
    }
    #[test]
    fn test_barycenter_trig_rotational_elements_nut_prec_angles_4() {
        assert_eq!(
            &[
                190.72646643f64,
                15917.10818695f64,
                0f64,
                21.4689247f64,
                31834.27934054f64,
                0f64,
                332.86082793f64,
                19139.89694742f64,
                0f64,
                394.93256437f64,
                38280.79631835f64,
                0f64,
                189.6327156f64,
                41215158.1842005f64,
                12.711923222f64,
                121.46893664f64,
                660.22803474f64,
                0f64,
                231.05028581f64,
                660.9912354f64,
                0f64,
                251.37314025f64,
                1320.50145245f64,
                0f64,
                217.98635955f64,
                38279.9612555f64,
                0f64,
                196.19729402f64,
                19139.83628608f64,
                0f64,
                198.991226f64,
                19139.4819985f64,
                0f64,
                226.292679f64,
                38280.8511281f64,
                0f64,
                249.663391f64,
                57420.7251593f64,
                0f64,
                266.18351f64,
                76560.636795f64,
                0f64,
                79.398797f64,
                0.5042615f64,
                0f64,
                122.433576f64,
                19139.9407476f64,
                0f64,
                43.058401f64,
                38280.8753272f64,
                0f64,
                57.663379f64,
                57420.7517205f64,
                0f64,
                79.476401f64,
                76560.6495004f64,
                0f64,
                166.325722f64,
                0.5042615f64,
                0f64,
                129.071773f64,
                19140.0328244f64,
                0f64,
                36.352167f64,
                38281.0473591f64,
                0f64,
                56.668646f64,
                57420.929536f64,
                0f64,
                67.364003f64,
                76560.2552215f64,
                0f64,
                104.79268f64,
                95700.4387578f64,
                0f64,
                95.391654f64,
                0.5042615f64,
                0f64
            ],
            MarsBarycenter::NUT_PREC_ANGLES
        )
    }
    #[test]
    fn test_naif_id_5() {
        assert_eq!(JupiterBarycenter::id(), 5i32)
    }
    #[test]
    fn test_point_mass_5() {
        assert_eq!(
            JupiterBarycenter::gravitational_parameter(),
            126712764.09999998f64
        );
    }
    #[test]
    fn test_barycenter_trig_rotational_elements_nut_prec_angles_5() {
        assert_eq!(
            &[
                73.32f64,
                91472.9f64,
                24.62f64,
                45137.2f64,
                283.9f64,
                4850.7f64,
                355.8f64,
                1191.3f64,
                119.9f64,
                262.1f64,
                229.8f64,
                64.3f64,
                352.25f64,
                2382.6f64,
                113.35f64,
                6070f64,
                146.64f64,
                182945.8f64,
                49.24f64,
                90274.4f64,
                99.360714f64,
                4850.4046f64,
                175.895369f64,
                1191.9605f64,
                300.323162f64,
                262.5475f64,
                114.012305f64,
                6070.2476f64,
                49.511251f64,
                64.3f64
            ],
            JupiterBarycenter::NUT_PREC_ANGLES
        )
    }
    #[test]
    fn test_naif_id_6() {
        assert_eq!(SaturnBarycenter::id(), 6i32)
    }
    #[test]
    fn test_point_mass_6() {
        assert_eq!(
            SaturnBarycenter::gravitational_parameter(),
            37940584.8418f64
        );
    }
    #[test]
    fn test_barycenter_trig_rotational_elements_nut_prec_angles_6() {
        assert_eq!(
            &[
                353.32f64,
                75706.7f64,
                28.72f64,
                75706.7f64,
                177.4f64,
                -36505.5f64,
                300f64,
                -7225.9f64,
                316.45f64,
                506.2f64,
                345.2f64,
                -1016.3f64,
                706.64f64,
                151413.4f64,
                57.44f64,
                151413.4f64
            ],
            SaturnBarycenter::NUT_PREC_ANGLES
        )
    }
    #[test]
    fn test_naif_id_7() {
        assert_eq!(UranusBarycenter::id(), 7i32)
    }
    #[test]
    fn test_point_mass_7() {
        assert_eq!(
            UranusBarycenter::gravitational_parameter(),
            5794556.3999999985f64
        );
    }
    #[test]
    fn test_barycenter_trig_rotational_elements_nut_prec_angles_7() {
        assert_eq!(
            &[
                115.75f64,
                54991.87f64,
                141.69f64,
                41887.66f64,
                135.03f64,
                29927.35f64,
                61.77f64,
                25733.59f64,
                249.32f64,
                24471.46f64,
                43.86f64,
                22278.41f64,
                77.66f64,
                20289.42f64,
                157.36f64,
                16652.76f64,
                101.81f64,
                12872.63f64,
                138.64f64,
                8061.81f64,
                102.23f64,
                -2024.22f64,
                316.41f64,
                2863.96f64,
                304.01f64,
                -51.94f64,
                308.71f64,
                -93.17f64,
                340.82f64,
                -75.32f64,
                259.14f64,
                -504.81f64,
                204.46f64,
                -4048.44f64,
                632.82f64,
                5727.92f64
            ],
            UranusBarycenter::NUT_PREC_ANGLES
        )
    }
    #[test]
    fn test_naif_id_8() {
        assert_eq!(NeptuneBarycenter::id(), 8i32)
    }
    #[test]
    fn test_point_mass_8() {
        assert_eq!(
            NeptuneBarycenter::gravitational_parameter(),
            6836527.100580399f64
        );
    }
    #[test]
    fn test_barycenter_trig_rotational_elements_nut_prec_angles_8() {
        assert_eq!(
            &[
                357.85f64,
                52.316f64,
                323.92f64,
                62606.6f64,
                220.51f64,
                55064.2f64,
                354.27f64,
                46564.5f64,
                75.31f64,
                26109.4f64,
                35.36f64,
                14325.4f64,
                142.61f64,
                2824.6f64,
                177.85f64,
                52.316f64,
                647.84f64,
                125213.2f64,
                355.7f64,
                104.632f64,
                533.55f64,
                156.948f64,
                711.4f64,
                209.264f64,
                889.25f64,
                261.58f64,
                1067.1f64,
                313.896f64,
                1244.95f64,
                366.212f64,
                1422.8f64,
                418.528f64,
                1600.65f64,
                470.844f64
            ],
            NeptuneBarycenter::NUT_PREC_ANGLES
        )
    }
    #[test]
    fn test_naif_id_9() {
        assert_eq!(PlutoBarycenter::id(), 9i32)
    }
    #[test]
    fn test_point_mass_9() {
        assert_eq!(PlutoBarycenter::gravitational_parameter(), 975.5f64);
    }
}
