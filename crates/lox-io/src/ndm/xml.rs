// SPDX-FileCopyrightText: 2024 Andrei Zisu <matzipan@gmail.com>
//
// SPDX-License-Identifier: MPL-2.0

//! The public interface for the XML deserializer type

pub(crate) mod error;

pub use error::XmlDeserializationError;

mod deserializer;

pub use deserializer::FromXmlStr;
