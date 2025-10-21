// SPDX-FileCopyrightText: 2024 Andrei Zisu <matzipan@gmail.com>
// SPDX-License-Identifier: MPL-2.0

use nom::error::ErrorKind;

pub trait KvnDeserializer {
    fn deserialize<'a>(
        lines: &mut std::iter::Peekable<impl Iterator<Item = &'a str>>,
    ) -> Result<Self, KvnDeserializerErr<String>>
    where
        Self: Sized;

    fn from_kvn_str(kvn: &str) -> Result<Self, KvnDeserializerErr<String>>
    where
        Self: Sized,
    {
        Self::deserialize(&mut kvn.lines().peekable())
    }

    fn should_check_key_match() -> bool;
}

#[derive(PartialEq, Clone, thiserror::Error, Debug)]
pub enum KvnDeserializerErr<I> {
    InvalidDateTimeFormat { input: I },
    InvalidNumberFormat { input: I },
    InvalidStringFormat { input: I },
    InvalidStateVectorFormat { input: I },
    InvalidCovarianceMatrixFormat { input: I },
    KeywordNotFound { expected: I },
    // Has a second meaning: it stops the iterator for vector type deserializers
    UnexpectedKeyword { found: I, expected: I },
    EmptyKeyword { input: I },
    EmptyValue { input: I },
    UnexpectedEndOfInput { keyword: I },
    GeneralParserError(I, ErrorKind),
}
