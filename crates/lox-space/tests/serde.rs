// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

#![cfg(feature = "serde")]

use lox_core::coords::Cartesian;
use lox_core::coords::LonLatAlt;
use lox_core::glam::{DMat3, DVec3};
use lox_space::bodies::*;
use lox_space::frames::Teme;
use lox_space::frames::rotations::Rotation;
use lox_space::frames::*;
use lox_space::orbits::CartesianOrbit;
use lox_space::orbits::events::ZeroCrossing;
use lox_space::orbits::ground::GroundLocation;
use lox_space::time::Time;
use lox_space::time::intervals::TimeInterval;
use lox_space::time::time_scales::Tai;
use lox_space::units::*;
use serde::de::DeserializeOwned;

fn round_trip<T>(value: &T) -> T
where
    T: serde::Serialize + DeserializeOwned + std::fmt::Debug + PartialEq,
{
    let json = serde_json::to_string(value).expect("serialize");
    let back: T = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(value, &back);
    back
}

// -- lox-core types --

#[test]
fn test_angle() {
    round_trip(&Angle::radians(1.23));
}

#[test]
fn test_distance() {
    round_trip(&Distance::meters(42.0));
}

#[test]
fn test_velocity() {
    round_trip(&Velocity::meters_per_second(7.8));
}

// -- lox-bodies types --

/// Generates a round-trip serde test for each body ZST, verifying that it
/// serializes as its name string (not `null`) and deserializes back.
macro_rules! body_serde_tests {
    ($($test_name:ident: $ty:ident => $name:literal),* $(,)?) => {
        $(
            #[test]
            fn $test_name() {
                let json = serde_json::to_string(&$ty).expect("serialize");
                assert_eq!(json, concat!("\"", $name, "\""));
                let back: $ty = serde_json::from_str(&json).expect("deserialize");
                assert_eq!(back, $ty);
            }
        )*
    };
}

body_serde_tests! {
    // Planets
    test_body_sun:                  Sun => "Sun",
    test_body_mercury:              Mercury => "Mercury",
    test_body_venus:                Venus => "Venus",
    test_body_earth:                Earth => "Earth",
    test_body_mars:                 Mars => "Mars",
    test_body_jupiter:              Jupiter => "Jupiter",
    test_body_saturn:               Saturn => "Saturn",
    test_body_uranus:               Uranus => "Uranus",
    test_body_neptune:              Neptune => "Neptune",
    test_body_pluto:                Pluto => "Pluto",
    // Barycenters
    test_body_ssb:                  SolarSystemBarycenter => "Solar System Barycenter",
    test_body_mercury_barycenter:   MercuryBarycenter => "Mercury Barycenter",
    test_body_venus_barycenter:     VenusBarycenter => "Venus Barycenter",
    test_body_earth_barycenter:     EarthBarycenter => "Earth Barycenter",
    test_body_mars_barycenter:      MarsBarycenter => "Mars Barycenter",
    test_body_jupiter_barycenter:   JupiterBarycenter => "Jupiter Barycenter",
    test_body_saturn_barycenter:    SaturnBarycenter => "Saturn Barycenter",
    test_body_uranus_barycenter:    UranusBarycenter => "Uranus Barycenter",
    test_body_neptune_barycenter:   NeptuneBarycenter => "Neptune Barycenter",
    test_body_pluto_barycenter:     PlutoBarycenter => "Pluto Barycenter",
    // Earth system
    test_body_moon:                 Moon => "Moon",
    // Mars system
    test_body_phobos:               Phobos => "Phobos",
    test_body_deimos:               Deimos => "Deimos",
    // Jupiter system
    test_body_io:                   Io => "Io",
    test_body_europa:               Europa => "Europa",
    test_body_ganymede:             Ganymede => "Ganymede",
    test_body_callisto:             Callisto => "Callisto",
    test_body_amalthea:             Amalthea => "Amalthea",
    test_body_himalia:              Himalia => "Himalia",
    test_body_elara:                Elara => "Elara",
    test_body_pasiphae:             Pasiphae => "Pasiphae",
    test_body_sinope:               Sinope => "Sinope",
    test_body_lysithea:             Lysithea => "Lysithea",
    test_body_carme:                Carme => "Carme",
    test_body_ananke:               Ananke => "Ananke",
    test_body_leda:                 Leda => "Leda",
    test_body_thebe:                Thebe => "Thebe",
    test_body_adrastea:             Adrastea => "Adrastea",
    test_body_metis:                Metis => "Metis",
    test_body_callirrhoe:           Callirrhoe => "Callirrhoe",
    test_body_themisto:             Themisto => "Themisto",
    test_body_magaclite:            Magaclite => "Magaclite",
    test_body_taygete:              Taygete => "Taygete",
    test_body_chaldene:             Chaldene => "Chaldene",
    test_body_harpalyke:            Harpalyke => "Harpalyke",
    test_body_kalyke:               Kalyke => "Kalyke",
    test_body_iocaste:              Iocaste => "Iocaste",
    test_body_erinome:              Erinome => "Erinome",
    test_body_isonoe:               Isonoe => "Isonoe",
    test_body_praxidike:            Praxidike => "Praxidike",
    test_body_autonoe:              Autonoe => "Autonoe",
    test_body_thyone:               Thyone => "Thyone",
    test_body_hermippe:             Hermippe => "Hermippe",
    test_body_aitne:                Aitne => "Aitne",
    test_body_eurydome:             Eurydome => "Eurydome",
    test_body_euanthe:              Euanthe => "Euanthe",
    test_body_euporie:              Euporie => "Euporie",
    test_body_orthosie:             Orthosie => "Orthosie",
    test_body_sponde:               Sponde => "Sponde",
    test_body_kale:                 Kale => "Kale",
    test_body_pasithee:             Pasithee => "Pasithee",
    test_body_hegemone:             Hegemone => "Hegemone",
    test_body_mneme:                Mneme => "Mneme",
    test_body_aoede:                Aoede => "Aoede",
    test_body_thelxinoe:            Thelxinoe => "Thelxinoe",
    test_body_arche:                Arche => "Arche",
    test_body_kallichore:           Kallichore => "Kallichore",
    test_body_helike:               Helike => "Helike",
    test_body_carpo:                Carpo => "Carpo",
    test_body_eukelade:             Eukelade => "Eukelade",
    test_body_cyllene:              Cyllene => "Cyllene",
    test_body_kore:                 Kore => "Kore",
    test_body_herse:                Herse => "Herse",
    test_body_dia:                  Dia => "Dia",
    // Saturn system
    test_body_mimas:                Mimas => "Mimas",
    test_body_enceladus:            Enceladus => "Enceladus",
    test_body_tethys:               Tethys => "Tethys",
    test_body_dione:                Dione => "Dione",
    test_body_rhea:                 Rhea => "Rhea",
    test_body_titan:                Titan => "Titan",
    test_body_hyperion:             Hyperion => "Hyperion",
    test_body_iapetus:              Iapetus => "Iapetus",
    test_body_phoebe:               Phoebe => "Phoebe",
    test_body_janus:                Janus => "Janus",
    test_body_epimetheus:           Epimetheus => "Epimetheus",
    test_body_helene:               Helene => "Helene",
    test_body_telesto:              Telesto => "Telesto",
    test_body_calypso:              Calypso => "Calypso",
    test_body_atlas:                Atlas => "Atlas",
    test_body_prometheus:           Prometheus => "Prometheus",
    test_body_pandora:              Pandora => "Pandora",
    test_body_pan:                  Pan => "Pan",
    test_body_ymir:                 Ymir => "Ymir",
    test_body_paaliaq:              Paaliaq => "Paaliaq",
    test_body_tarvos:               Tarvos => "Tarvos",
    test_body_ijiraq:               Ijiraq => "Ijiraq",
    test_body_suttungr:             Suttungr => "Suttungr",
    test_body_kiviuq:               Kiviuq => "Kiviuq",
    test_body_mundilfari:           Mundilfari => "Mundilfari",
    test_body_albiorix:             Albiorix => "Albiorix",
    test_body_skathi:               Skathi => "Skathi",
    test_body_erriapus:             Erriapus => "Erriapus",
    test_body_siarnaq:              Siarnaq => "Siarnaq",
    test_body_thrymr:               Thrymr => "Thrymr",
    test_body_narvi:                Narvi => "Narvi",
    test_body_methone:              Methone => "Methone",
    test_body_pallene:              Pallene => "Pallene",
    test_body_polydeuces:           Polydeuces => "Polydeuces",
    test_body_daphnis:              Daphnis => "Daphnis",
    test_body_aegir:                Aegir => "Aegir",
    test_body_bebhionn:             Bebhionn => "Bebhionn",
    test_body_bergelmir:            Bergelmir => "Bergelmir",
    test_body_bestla:               Bestla => "Bestla",
    test_body_farbauti:             Farbauti => "Farbauti",
    test_body_fenrir:               Fenrir => "Fenrir",
    test_body_fornjot:              Fornjot => "Fornjot",
    test_body_hati:                 Hati => "Hati",
    test_body_hyrrokkin:            Hyrrokkin => "Hyrrokkin",
    test_body_kari:                 Kari => "Kari",
    test_body_loge:                 Loge => "Loge",
    test_body_skoll:                Skoll => "Skoll",
    test_body_surtur:               Surtur => "Surtur",
    test_body_anthe:                Anthe => "Anthe",
    test_body_jarnsaxa:             Jarnsaxa => "Jarnsaxa",
    test_body_greip:                Greip => "Greip",
    test_body_tarqeq:               Tarqeq => "Tarqeq",
    test_body_aegaeon:              Aegaeon => "Aegaeon",
    // Uranus system
    test_body_ariel:                Ariel => "Ariel",
    test_body_umbriel:              Umbriel => "Umbriel",
    test_body_titania:              Titania => "Titania",
    test_body_oberon:               Oberon => "Oberon",
    test_body_miranda:              Miranda => "Miranda",
    test_body_cordelia:             Cordelia => "Cordelia",
    test_body_ophelia:              Ophelia => "Ophelia",
    test_body_bianca:               Bianca => "Bianca",
    test_body_cressida:             Cressida => "Cressida",
    test_body_desdemona:            Desdemona => "Desdemona",
    test_body_juliet:               Juliet => "Juliet",
    test_body_portia:               Portia => "Portia",
    test_body_rosalind:             Rosalind => "Rosalind",
    test_body_belinda:              Belinda => "Belinda",
    test_body_puck:                 Puck => "Puck",
    test_body_caliban:              Caliban => "Caliban",
    test_body_sycorax:              Sycorax => "Sycorax",
    test_body_prospero:             Prospero => "Prospero",
    test_body_setebos:              Setebos => "Setebos",
    test_body_stephano:             Stephano => "Stephano",
    test_body_trinculo:             Trinculo => "Trinculo",
    test_body_francisco:            Francisco => "Francisco",
    test_body_margaret:             Margaret => "Margaret",
    test_body_ferdinand:            Ferdinand => "Ferdinand",
    test_body_perdita:              Perdita => "Perdita",
    test_body_mab:                  Mab => "Mab",
    test_body_cupid:                Cupid => "Cupid",
    // Neptune system
    test_body_triton:               Triton => "Triton",
    test_body_nereid:               Nereid => "Nereid",
    test_body_naiad:                Naiad => "Naiad",
    test_body_thalassa:             Thalassa => "Thalassa",
    test_body_despina:              Despina => "Despina",
    test_body_galatea:              Galatea => "Galatea",
    test_body_larissa:              Larissa => "Larissa",
    test_body_proteus:              Proteus => "Proteus",
    test_body_halimede:             Halimede => "Halimede",
    test_body_psamathe:             Psamathe => "Psamathe",
    test_body_sao:                  Sao => "Sao",
    test_body_laomedeia:            Laomedeia => "Laomedeia",
    test_body_neso:                 Neso => "Neso",
    // Pluto system
    test_body_charon:               Charon => "Charon",
    test_body_nix:                  Nix => "Nix",
    test_body_hydra:                Hydra => "Hydra",
    test_body_kerberos:             Kerberos => "Kerberos",
    test_body_styx:                 Styx => "Styx",
    // Minor bodies
    test_body_gaspra:               Gaspra => "Gaspra",
    test_body_ida:                  Ida => "Ida",
    test_body_dactyl:               Dactyl => "Dactyl",
    test_body_ceres:                Ceres => "Ceres",
    test_body_pallas:               Pallas => "Pallas",
    test_body_vesta:                Vesta => "Vesta",
    test_body_psyche:               Psyche => "Psyche",
    test_body_lutetia:              Lutetia => "Lutetia",
    test_body_kleopatra:            Kleopatra => "Kleopatra",
    test_body_eros:                 Eros => "Eros",
    test_body_davida:               Davida => "Davida",
    test_body_mathilde:             Mathilde => "Mathilde",
    test_body_steins:               Steins => "Steins",
    test_body_braille:              Braille => "Braille",
    test_body_wilson_harrington:    WilsonHarrington => "Wilson-Harrington",
    test_body_toutatis:             Toutatis => "Toutatis",
    test_body_itokawa:              Itokawa => "Itokawa",
    test_body_bennu:                Bennu => "Bennu",
}

#[test]
fn test_naif_id() {
    round_trip(&NaifId(399));
}

#[test]
fn test_dyn_origin() {
    round_trip(&DynOrigin::Earth);
    round_trip(&DynOrigin::Moon);
    round_trip(&DynOrigin::Sun);
}

// -- lox-time types --

#[test]
fn test_time_tai() {
    let t = Time::new(Tai, 0, Default::default());
    round_trip(&t);
}

#[test]
fn test_time_with_offset() {
    let t = Time::new(Tai, 86400, Default::default());
    round_trip(&t);
}

#[test]
fn test_time_scale_zsts_serialize_as_abbreviation() {
    use lox_space::time::time_scales::{Tcb, Tcg, Tdb, Tt, Ut1};

    assert_eq!(serde_json::to_string(&Tai).unwrap(), "\"TAI\"");
    assert_eq!(serde_json::to_string(&Tcb).unwrap(), "\"TCB\"");
    assert_eq!(serde_json::to_string(&Tcg).unwrap(), "\"TCG\"");
    assert_eq!(serde_json::to_string(&Tdb).unwrap(), "\"TDB\"");
    assert_eq!(serde_json::to_string(&Tt).unwrap(), "\"TT\"");
    assert_eq!(serde_json::to_string(&Ut1).unwrap(), "\"UT1\"");
}

#[test]
fn test_time_scale_zsts_round_trip() {
    use lox_space::time::time_scales::{Tcb, Tcg, Tdb, Tt, Ut1};

    round_trip(&Tai);
    round_trip(&Tcb);
    round_trip(&Tcg);
    round_trip(&Tdb);
    round_trip(&Tt);
    round_trip(&Ut1);
}

// -- lox-frames types --

#[test]
fn test_frame_zsts() {
    round_trip(&Icrf);
    round_trip(&Itrf);
    round_trip(&Cirf);
    round_trip(&Tirf);
    round_trip(&Teme);
}

#[test]
fn test_frame_zsts_serialize_as_abbreviation() {
    assert_eq!(serde_json::to_string(&Icrf).unwrap(), "\"ICRF\"");
    assert_eq!(serde_json::to_string(&Itrf).unwrap(), "\"ITRF\"");
    assert_eq!(serde_json::to_string(&Cirf).unwrap(), "\"CIRF\"");
    assert_eq!(serde_json::to_string(&Tirf).unwrap(), "\"TIRF\"");
    assert_eq!(serde_json::to_string(&Teme).unwrap(), "\"TEME\"");
}

#[test]
fn test_dyn_frame() {
    round_trip(&DynFrame::Icrf);
    round_trip(&DynFrame::Itrf);
    round_trip(&DynFrame::Iau(DynOrigin::Earth));
}

#[test]
fn test_rotation() {
    let r = Rotation {
        m: DMat3::IDENTITY,
        dm: DMat3::ZERO,
    };
    round_trip(&r);
}

// -- lox-orbits types --

#[test]
fn test_state() {
    let t = Time::new(Tai, 0, Default::default());
    let pos = DVec3::new(6778.0, 0.0, 0.0);
    let vel = DVec3::new(0.0, 7.5, 0.0);
    let state = CartesianOrbit::new(Cartesian::from_vecs(pos, vel), t, Earth, Icrf);
    round_trip(&state);
}

#[test]
fn test_ground_location() {
    let loc = GroundLocation::new(LonLatAlt::default(), Earth);
    let json = serde_json::to_string(&loc).expect("serialize");
    let _: GroundLocation<Earth> = serde_json::from_str(&json).expect("deserialize");
}

#[test]
fn test_time_interval() {
    let t0 = Time::new(Tai, 0, Default::default());
    let t1 = Time::new(Tai, 3600, Default::default());
    let i = TimeInterval::new(t0, t1);
    round_trip(&i);
}

#[test]
fn test_zero_crossing() {
    round_trip(&ZeroCrossing::Up);
    round_trip(&ZeroCrossing::Down);
}

// -- lox-orbits propagator & trajectory types --

fn round_trip_no_eq<T>(value: &T)
where
    T: serde::Serialize + DeserializeOwned + std::fmt::Debug,
{
    let json = serde_json::to_string(value).expect("serialize");
    let _: T = serde_json::from_str(&json).expect("deserialize");
}

#[test]
fn test_vallado() {
    use lox_space::orbits::propagators::semi_analytical::Vallado;

    let t = Time::new(Tai, 0, Default::default());
    let orbit = CartesianOrbit::new(
        Cartesian::from_vecs(
            DVec3::new(6_778_000.0, 0.0, 0.0),
            DVec3::new(0.0, 7500.0, 0.0),
        ),
        t,
        Earth,
        Icrf,
    );
    let v = Vallado::new(orbit);
    round_trip(&v);
}

#[test]
fn test_numerical_propagator() {
    use lox_space::orbits::propagators::numerical::NumericalPropagator;

    let t = Time::new(Tai, 0, Default::default());
    let orbit = CartesianOrbit::new(
        Cartesian::from_vecs(
            DVec3::new(6_778_000.0, 0.0, 0.0),
            DVec3::new(0.0, 7500.0, 0.0),
        ),
        t,
        Earth,
        Icrf,
    );
    let p = NumericalPropagator::new(orbit);
    round_trip_no_eq(&p);
}

#[test]
fn test_j2_propagator() {
    use lox_space::orbits::propagators::j2::J2Propagator;

    let t = Time::new(Tai, 0, Default::default());
    let orbit = CartesianOrbit::new(
        Cartesian::from_vecs(
            DVec3::new(6_778_000.0, 0.0, 0.0),
            DVec3::new(0.0, 7500.0, 0.0),
        ),
        t,
        Earth,
        Icrf,
    );
    let p = J2Propagator::try_new(orbit).unwrap();
    round_trip_no_eq(&p);
}

#[test]
fn test_j4_propagator() {
    use lox_space::orbits::propagators::j4::J4Propagator;

    let t = Time::new(Tai, 0, Default::default());
    let orbit = CartesianOrbit::new(
        Cartesian::from_vecs(
            DVec3::new(6_778_000.0, 0.0, 0.0),
            DVec3::new(0.0, 7500.0, 0.0),
        ),
        t,
        Earth,
        Icrf,
    );
    let p = J4Propagator::try_new(orbit).unwrap();
    round_trip_no_eq(&p);
}

#[test]
fn test_sgp4() {
    use lox_space::orbits::propagators::sgp4::{Elements, Sgp4};

    let tle = Elements::from_tle(
        Some("ISS (ZARYA)".to_string()),
        "1 25544U 98067A   24170.37528350  .00016566  00000+0  30244-3 0  9996".as_bytes(),
        "2 25544  51.6410 309.3890 0010444 339.5369 107.8830 15.49495945458731".as_bytes(),
    )
    .unwrap();
    let sgp4 = Sgp4::new(tle).unwrap();
    round_trip_no_eq(&sgp4);
}

#[test]
fn test_trajectory() {
    use lox_space::orbits::DynTrajectory;

    let traj = DynTrajectory::from_csv_dyn(
        &lox_test_utils::read_data_file("trajectory_lunar.csv"),
        DynOrigin::Earth,
        DynFrame::Icrf,
    )
    .unwrap();
    round_trip_no_eq(&traj);
}

#[test]
fn test_orbit_source_trajectory() {
    use lox_space::orbits::DynTrajectory;
    use lox_space::orbits::propagators::OrbitSource;

    let traj = DynTrajectory::from_csv_dyn(
        &lox_test_utils::read_data_file("trajectory_lunar.csv"),
        DynOrigin::Earth,
        DynFrame::Icrf,
    )
    .unwrap();
    round_trip_no_eq(&OrbitSource::Trajectory(traj));
}

#[test]
fn test_orbit_source_vallado() {
    use lox_space::orbits::propagators::OrbitSource;
    use lox_space::orbits::propagators::semi_analytical::DynVallado;

    let t = Time::new(Tai, 0, Default::default());
    let orbit = CartesianOrbit::new(
        Cartesian::from_vecs(
            DVec3::new(6_778_000.0, 0.0, 0.0),
            DVec3::new(0.0, 7500.0, 0.0),
        ),
        t,
        Earth,
        Icrf,
    );
    let v = DynVallado::try_new(orbit.into_dyn()).unwrap();
    round_trip_no_eq(&OrbitSource::Vallado(v));
}

#[test]
fn test_orbit_source_sgp4() {
    use lox_space::orbits::propagators::OrbitSource;
    use lox_space::orbits::propagators::sgp4::{Elements, Sgp4};

    let tle = Elements::from_tle(
        Some("ISS (ZARYA)".to_string()),
        "1 25544U 98067A   24170.37528350  .00016566  00000+0  30244-3 0  9996".as_bytes(),
        "2 25544  51.6410 309.3890 0010444 339.5369 107.8830 15.49495945458731".as_bytes(),
    )
    .unwrap();
    let sgp4 = Sgp4::new(tle).unwrap();
    round_trip_no_eq(&OrbitSource::Sgp4(sgp4));
}

#[test]
fn test_orbit_source_numerical() {
    use lox_space::orbits::propagators::OrbitSource;
    use lox_space::orbits::propagators::numerical::DynNumericalPropagator;

    let t = Time::new(Tai, 0, Default::default());
    let orbit = CartesianOrbit::new(
        Cartesian::from_vecs(
            DVec3::new(6_778_000.0, 0.0, 0.0),
            DVec3::new(0.0, 7500.0, 0.0),
        ),
        t,
        Earth,
        Icrf,
    );
    let p = DynNumericalPropagator::try_new(orbit.into_dyn()).unwrap();
    round_trip_no_eq(&OrbitSource::Numerical(p));
}

#[test]
fn test_orbit_source_j2() {
    use lox_space::orbits::propagators::OrbitSource;
    use lox_space::orbits::propagators::j2::DynJ2Propagator;

    let t = Time::new(Tai, 0, Default::default());
    let orbit = CartesianOrbit::new(
        Cartesian::from_vecs(
            DVec3::new(6_778_000.0, 0.0, 0.0),
            DVec3::new(0.0, 7500.0, 0.0),
        ),
        t,
        Earth,
        Icrf,
    );
    let p = DynJ2Propagator::try_new(orbit.into_dyn()).unwrap();
    round_trip_no_eq(&OrbitSource::J2(p));
}

#[test]
fn test_orbit_source_j4() {
    use lox_space::orbits::propagators::OrbitSource;
    use lox_space::orbits::propagators::j4::DynJ4Propagator;

    let t = Time::new(Tai, 0, Default::default());
    let orbit = CartesianOrbit::new(
        Cartesian::from_vecs(
            DVec3::new(6_778_000.0, 0.0, 0.0),
            DVec3::new(0.0, 7500.0, 0.0),
        ),
        t,
        Earth,
        Icrf,
    );
    let p = DynJ4Propagator::try_new(orbit.into_dyn()).unwrap();
    round_trip_no_eq(&OrbitSource::J4(p));
}

// -- lox-orbits constellation types --

#[test]
fn test_constellation_propagator() {
    use lox_space::orbits::constellations::ConstellationPropagator;

    round_trip(&ConstellationPropagator::Vallado);
    round_trip(&ConstellationPropagator::Numerical);
    round_trip(&ConstellationPropagator::J2);
    round_trip(&ConstellationPropagator::J2Osc);
    round_trip(&ConstellationPropagator::J4);
    round_trip(&ConstellationPropagator::J4Osc);
}

#[test]
fn test_constellation_satellite() {
    use lox_space::orbits::constellations::WalkerDeltaBuilder;

    let t = Time::new(Tai, 0, Default::default());
    let constellation = WalkerDeltaBuilder::new(6, 3)
        .with_semi_major_axis(Distance::kilometers(7000.0), 0.0)
        .with_inclination(Angle::degrees(53.0))
        .build_constellation("test", t, Earth, Icrf)
        .unwrap();
    let sat = &constellation.satellites()[0];
    round_trip(sat);
}

#[test]
fn test_constellation() {
    use lox_space::orbits::constellations::WalkerDeltaBuilder;

    let t = Time::new(Tai, 0, Default::default());
    let constellation = WalkerDeltaBuilder::new(6, 3)
        .with_semi_major_axis(Distance::kilometers(7000.0), 0.0)
        .with_inclination(Angle::degrees(53.0))
        .build_constellation("test", t, Earth, Icrf)
        .unwrap();
    round_trip_no_eq(&constellation);
}

// -- lox-analysis types --

#[test]
fn test_asset_id() {
    use lox_space::analysis::assets::AssetId;
    round_trip(&AssetId::new("station-1"));
}

#[test]
fn test_constellation_id() {
    use lox_space::analysis::assets::ConstellationId;
    round_trip(&ConstellationId::new("oneweb"));
}

#[test]
fn test_network_id() {
    use lox_space::analysis::assets::NetworkId;
    round_trip(&NetworkId::new("estrack"));
}

#[test]
fn test_elevation_mask_fixed() {
    use lox_space::analysis::visibility::ElevationMask;
    round_trip(&ElevationMask::with_fixed_elevation(0.1));
}

#[test]
fn test_elevation_mask_variable() {
    use lox_space::analysis::visibility::ElevationMask;
    use std::f64::consts::PI;
    let mask = ElevationMask::new(
        vec![-PI, -PI / 2.0, 0.0, PI / 2.0, PI],
        vec![5.0, 10.0, 5.0, 10.0, 5.0],
    )
    .unwrap();
    round_trip(&mask);
}

#[test]
fn test_optical_payload() {
    use lox_space::analysis::imaging::OpticalPayload;
    round_trip_no_eq(&OpticalPayload::nadir_only(Distance::kilometers(50.0)));
    round_trip_no_eq(&OpticalPayload::off_nadir(
        Distance::kilometers(50.0),
        Angle::degrees(30.0),
    ));
}

#[test]
fn test_sar_payload() {
    use lox_space::analysis::imaging::{LookSide, SarPayload};
    for side in [LookSide::Left, LookSide::Right, LookSide::Either] {
        round_trip_no_eq(
            &SarPayload::with_look_angles(Angle::degrees(20.0), Angle::degrees(45.0), side)
                .unwrap(),
        );
        round_trip_no_eq(
            &SarPayload::with_incidence_angles(Angle::degrees(29.0), Angle::degrees(46.0), side)
                .unwrap(),
        );
    }
}

#[test]
fn test_access_window() {
    use lox_space::analysis::imaging::{AccessWindow, PassDirection};
    use lox_space::time::deltas::TimeDelta;

    let start = Time::j2000(Tai);
    let end = start + TimeDelta::from_seconds(120);
    let interval = TimeInterval::new(start, end);

    round_trip_no_eq(&AccessWindow {
        interval,
        direction: PassDirection::Ascending,
    });
    round_trip_no_eq(&AccessWindow {
        interval,
        direction: PassDirection::Descending,
    });
}

#[test]
fn test_ground_station() {
    use lox_space::analysis::assets::GroundStation;
    use lox_space::analysis::visibility::ElevationMask;
    use lox_space::orbits::ground::GroundLocation;

    let coords = LonLatAlt::from_degrees(-4.3676, 40.4527, 0.0).unwrap();
    let loc = GroundLocation::try_new(coords, DynOrigin::Earth).unwrap();
    let mask = ElevationMask::with_fixed_elevation(5.0);
    let gs = GroundStation::new("madrid", loc, mask).with_network_id("estrack");
    round_trip_no_eq(&gs);
}

#[test]
fn test_spacecraft() {
    use lox_space::analysis::assets::Spacecraft;
    use lox_space::orbits::DynTrajectory;
    use lox_space::orbits::propagators::OrbitSource;

    let traj = DynTrajectory::from_csv_dyn(
        &lox_test_utils::read_data_file("trajectory_lunar.csv"),
        DynOrigin::Earth,
        DynFrame::Icrf,
    )
    .unwrap();
    let sc = Spacecraft::new("sc-1", OrbitSource::Trajectory(traj)).with_constellation_id("test");
    round_trip_no_eq(&sc);
}

#[test]
fn test_spacecraft_with_payloads() {
    use lox_space::analysis::assets::Spacecraft;
    use lox_space::analysis::imaging::{LookSide, OpticalPayload, SarPayload};
    use lox_space::orbits::DynTrajectory;
    use lox_space::orbits::propagators::OrbitSource;

    let traj = DynTrajectory::from_csv_dyn(
        &lox_test_utils::read_data_file("trajectory_lunar.csv"),
        DynOrigin::Earth,
        DynFrame::Icrf,
    )
    .unwrap();
    let optical = OpticalPayload::off_nadir(Distance::kilometers(50.0), Angle::degrees(30.0));
    let sar = SarPayload::with_incidence_angles(
        Angle::degrees(29.0),
        Angle::degrees(46.0),
        LookSide::Right,
    )
    .unwrap();
    let sc = Spacecraft::new("sc-1", OrbitSource::Trajectory(traj))
        .with_optical_payload(optical)
        .with_sar_payload(sar);
    round_trip_no_eq(&sc);
}

#[test]
fn test_scenario() {
    use lox_space::analysis::assets::{GroundStation, Scenario, Spacecraft};
    use lox_space::analysis::visibility::ElevationMask;
    use lox_space::orbits::DynTrajectory;
    use lox_space::orbits::ground::GroundLocation;
    use lox_space::orbits::propagators::OrbitSource;

    let t0 = Time::new(Tai, 0, Default::default());
    let t1 = Time::new(Tai, 86400, Default::default());

    let coords = LonLatAlt::from_degrees(-4.3676, 40.4527, 0.0).unwrap();
    let loc = GroundLocation::try_new(coords, DynOrigin::Earth).unwrap();
    let gs = GroundStation::new("madrid", loc, ElevationMask::with_fixed_elevation(5.0));

    let traj = DynTrajectory::from_csv_dyn(
        &lox_test_utils::read_data_file("trajectory_lunar.csv"),
        DynOrigin::Earth,
        DynFrame::Icrf,
    )
    .unwrap();
    let sc = Spacecraft::new("sc-1", OrbitSource::Trajectory(traj));

    let scenario = Scenario::new(t0, t1, DynOrigin::Earth, DynFrame::Icrf)
        .with_ground_stations(&[gs])
        .with_spacecraft(&[sc]);
    round_trip_no_eq(&scenario);
}

// -- Verify JSON structure is human-readable --

#[test]
fn test_state_json_structure() {
    let t = Time::new(Tai, 100, Default::default());
    let state = CartesianOrbit::new(
        Cartesian::from_vecs(DVec3::new(6778.0, 0.0, 0.0), DVec3::new(0.0, 7.5, 0.0)),
        t,
        Earth,
        Icrf,
    );
    let json: serde_json::Value = serde_json::to_value(state).expect("serialize");
    assert!(json.get("time").is_some());
    assert!(json.get("state").is_some());
    assert_eq!(json.get("origin").unwrap(), "Earth");
    assert_eq!(json.get("frame").unwrap(), "ICRF");
}

#[test]
fn test_scenario_json_structure() {
    use lox_space::analysis::assets::Scenario;

    let t0 = Time::new(Tai, 0, Default::default());
    let t1 = Time::new(Tai, 86400, Default::default());
    let scenario = Scenario::new(t0, t1, Earth, Icrf);
    let json: serde_json::Value = serde_json::to_value(scenario).expect("serialize");
    assert_eq!(json.get("origin").unwrap(), "Earth");
    assert_eq!(json.get("frame").unwrap(), "ICRF");
    assert!(json.get("interval").is_some());
    assert!(json.get("ground_stations").unwrap().is_array());
    assert!(json.get("spacecraft").unwrap().is_array());
    assert!(json.get("constellations").unwrap().is_array());
}

#[test]
fn test_scenario_dyn_json_structure() {
    use lox_space::analysis::assets::Scenario;

    let t0 = Time::new(Tai, 0, Default::default());
    let t1 = Time::new(Tai, 86400, Default::default());
    let scenario = Scenario::new(t0, t1, DynOrigin::Earth, DynFrame::Icrf);
    let json: serde_json::Value = serde_json::to_value(scenario).expect("serialize");
    assert_eq!(json.get("origin").unwrap(), "Earth");
    assert_eq!(json.get("frame").unwrap(), "Icrf");
}

#[test]
fn test_time_json_structure() {
    let t = Time::new(Tai, 100, Default::default());
    let json: serde_json::Value = serde_json::to_value(t).expect("serialize");
    assert_eq!(json.get("scale").unwrap(), "TAI");
}
