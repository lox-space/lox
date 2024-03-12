/*
 * Copyright (c) 2023-2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum LoxBodiesError {
    #[error("unknown body `{0}`")]
    UnknownBody(String),
}
