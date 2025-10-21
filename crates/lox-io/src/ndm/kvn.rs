// SPDX-FileCopyrightText: 2024 Andrei Zisu <matzipan@gmail.com>
// SPDX-License-Identifier: MPL-2.0

//! The public interface for the `KvnDeserializer` type

mod deserializer;
pub(crate) mod parser;

pub use deserializer::{KvnDeserializer, KvnDeserializerErr};
