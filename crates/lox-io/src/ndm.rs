//! Parsers for CCSDS
//! [Navigation Data Messages version 3](https://public.ccsds.org/Pubs/502x0b3e1.pdf).
//!
//! The data types used to deserialize XML and KVN are generated from the CCSDS
//! NDM schema description. Some slight alterations have been made on top to
//! improve user friendliness and compatibility.
//!  
//! Since Rust XML and JSON deserializers are slightly incompatible, the JSON
//! deserializer is separate.
//!
//! Because there are a signficant number of messages out there that do not
//! strictly comply with the specification, the parsers are relaxed in terms of
//! input that they accept. Some relaxations:
//!
//! - The KVN floating point numbers defined in the specification can have only
//!   one character in the integer part of the number, but we accept any regular
//!   float number.
//! - The KVN strings are defined in the specification as being either only
//!   lower-case or only upper-case, but we accept any combination of cases.
//!
//! The XML deserializer does not perform any validation on the schema types
//! defined (e.g. non-positive double, lat-long, angle). The validation only
//! checks if the data can be parsed into the fundamental Rust data types
//! (e.g. f64, u64).
//!
//! The KVN parsing is implemented with a finite-state parser. As such, it is
//! eager and has no backtracking. This is normally okay. But the KVN grammar
//! is ambiguous with regards to its `COMMENT` fields, so in some corner-cases
//! it can lead to some `COMMENT` lines being discarded. This happens when an
//! optional section is ommitted. For example, an OPM message can have an empty
//! covariance matrix section. But this will cause the comments for the
//! following section, the maneuver parameters list, to be discarded.
//!
//! The KVN parsing currently does not support user-defined fields and ddd date
//! format types.
//!
//! Check the respective submodules for more information.

pub mod json;
pub mod kvn;
pub mod xml;

pub mod common;
pub mod ndm_ci;
pub mod ocm;
pub mod oem;
pub mod omm;
pub mod opm;
