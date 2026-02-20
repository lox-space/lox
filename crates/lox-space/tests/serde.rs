// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

#![cfg(feature = "serde")]

use glam::{DMat3, DVec3};
use lox_core::coords::Cartesian;
use lox_space::bodies::*;
use lox_space::frames::frames::Teme;
use lox_space::frames::rotations::Rotation;
use lox_space::frames::*;
use lox_space::orbits::events::{Window, ZeroCrossing};
use lox_space::orbits::ground::GroundLocation;
use lox_space::orbits::orbits::CartesianOrbit;
use lox_space::time::Time;
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

#[test]
fn test_body_zsts() {
    round_trip(&Earth);
    round_trip(&Moon);
    round_trip(&Sun);
    round_trip(&Mars);
    round_trip(&Jupiter);
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
    let loc = GroundLocation::new(0.0, 0.0, 0.0, Earth);
    let json = serde_json::to_string(&loc).expect("serialize");
    let _: GroundLocation<Earth> = serde_json::from_str(&json).expect("deserialize");
}

#[test]
fn test_window() {
    let t0 = Time::new(Tai, 0, Default::default());
    let t1 = Time::new(Tai, 3600, Default::default());
    let w = Window::new(t0, t1);
    round_trip(&w);
}

#[test]
fn test_zero_crossing() {
    round_trip(&ZeroCrossing::Up);
    round_trip(&ZeroCrossing::Down);
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
    let json: serde_json::Value = serde_json::to_value(&state).expect("serialize");
    assert!(json.get("time").is_some());
    assert!(json.get("state").is_some());
    assert!(json.get("origin").is_some());
    assert!(json.get("frame").is_some());
}
