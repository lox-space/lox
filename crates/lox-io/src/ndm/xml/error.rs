/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

/// Deserialization error. Simple wrapper for
/// quick_xml::errors::serialize::DeError. Check the source class for further
/// documentation.
#[derive(Clone, Debug)]
pub enum XmlDeserializationError {
    Custom(String),
    InvalidXml(String),
    InvalidInt(String),
    InvalidFloat(String),
    InvalidBoolean(String),
    KeyNotRead(String),
    UnexpectedStart(String),
    UnexpectedEnd(String),
    UnexpectedEof(String),
    ExpectedStart(String),
    Unsupported(String),
}

impl std::fmt::Display for XmlDeserializationError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            XmlDeserializationError::Custom(s) => write!(f, "{}", s),
            XmlDeserializationError::InvalidXml(s) => write!(f, "{}", s),
            XmlDeserializationError::InvalidInt(s) => write!(f, "{}", s),
            XmlDeserializationError::InvalidFloat(s) => write!(f, "{}", s),
            XmlDeserializationError::InvalidBoolean(s) => write!(f, "{}", s),
            XmlDeserializationError::KeyNotRead(s) => write!(f, "{}", s),
            XmlDeserializationError::UnexpectedStart(s) => write!(f, "{}", s),
            XmlDeserializationError::UnexpectedEnd(s) => write!(f, "{}", s),
            XmlDeserializationError::UnexpectedEof(s) => write!(f, "{}", s),
            XmlDeserializationError::ExpectedStart(s) => write!(f, "{}", s),
            XmlDeserializationError::Unsupported(s) => write!(f, "{}", s),
        }
    }
}

impl ::std::error::Error for XmlDeserializationError {}

impl From<quick_xml::DeError> for XmlDeserializationError {
    fn from(value: quick_xml::DeError) -> Self {
        let error_description = format!("{:?}", value).to_string();

        match value {
            quick_xml::DeError::Custom(_) => XmlDeserializationError::Custom(error_description),
            quick_xml::DeError::InvalidXml(_) => {
                XmlDeserializationError::InvalidXml(error_description)
            }
            quick_xml::DeError::InvalidInt(_) => {
                XmlDeserializationError::InvalidInt(error_description)
            }
            quick_xml::DeError::InvalidFloat(_) => {
                XmlDeserializationError::InvalidFloat(error_description)
            }
            quick_xml::DeError::InvalidBoolean(_) => {
                XmlDeserializationError::InvalidBoolean(error_description)
            }
            quick_xml::DeError::KeyNotRead => {
                XmlDeserializationError::KeyNotRead(error_description)
            }
            quick_xml::DeError::UnexpectedStart(_) => {
                XmlDeserializationError::UnexpectedStart(error_description)
            }
            quick_xml::DeError::UnexpectedEnd(_) => {
                XmlDeserializationError::UnexpectedEnd(error_description)
            }
            quick_xml::DeError::UnexpectedEof => {
                XmlDeserializationError::UnexpectedEof(error_description)
            }
            quick_xml::DeError::ExpectedStart => {
                XmlDeserializationError::ExpectedStart(error_description)
            }
            quick_xml::DeError::Unsupported(_) => {
                XmlDeserializationError::Unsupported(error_description)
            }
        }
    }
}
