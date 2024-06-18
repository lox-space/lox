/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

/// Parse an NDM message from a string formatted in XML
pub trait FromXmlStr<'a>: Sized + serde::Deserialize<'a> {
    fn from_xml_str(xml: &'a str) -> Result<Self, super::XmlDeserializationError> {
        Ok(quick_xml::de::from_str(xml)?)
    }
}
