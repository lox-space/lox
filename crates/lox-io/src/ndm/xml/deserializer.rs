// SPDX-FileCopyrightText: 2023 Andrei Zisu <matzipan@gmail.com>
// SPDX-FileCopyrightText: 2023 Angus Morrison <github@angus-morrison.com>
// SPDX-FileCopyrightText: 2023 Helge Eichhorn <git@helgeeichhorn.de>
// SPDX-License-Identifier: MPL-2.0

pub trait FromXmlStr<'a>: Sized + serde::Deserialize<'a> {
    fn from_xml_str(xml: &'a str) -> Result<Self, super::XmlDeserializationError> {
        Ok(quick_xml::de::from_str(xml)?)
    }
}
