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
//! Check the respective submodules for more information.

pub mod json;
pub mod kvn;

pub mod common;
pub mod ndm_ci;
pub mod ocm;
pub mod oem;
pub mod omm;
pub mod opm;
