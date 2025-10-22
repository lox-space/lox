// SPDX-FileCopyrightText: 2024 Andrei Zisu <matzipan@gmail.com>
//
// SPDX-License-Identifier: MPL-2.0

//! Deserializers for some JSON message types
//!
//! JSON message types are not fully standardized at CCSDS level. But they are
//! sometimes used in the wild due to the abundant availability of parsers.
//!
//! Only OMM messages were found to be used in the wild, so it is the only one
//! implemented.
//!
//! The data type is manually derived from the XML schema.

pub mod omm;
