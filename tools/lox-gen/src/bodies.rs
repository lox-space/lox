/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use proc_macro2::Ident;
use quote::format_ident;

pub struct BodyDef {
    pub name: &'static str,
    pub id: i32,
    pub body_trait: Option<&'static str>,
}

impl BodyDef {
    pub fn ident(&self) -> Ident {
        format_ident!("{}", self.name.replace([' ', '-'], ""))
    }

    pub fn body_trait(&self) -> Option<Ident> {
        self.body_trait
            .map(|body_trait| format_ident!("{}", body_trait))
    }
}
