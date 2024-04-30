/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use lox_io::spice::Kernel;

#[test]
fn test_lsk() {
    let lsk = include_str!("../../../data/naif0012.tls");
    let kernel = Kernel::from_string(lsk).expect("file should be parsable");
    assert_eq!(kernel.type_id(), "LSK");

    assert!(!kernel.keys().is_empty());

    let exp = vec![
        "10",
        "1972-JAN-1",
        "11",
        "1972-JUL-1",
        "12",
        "1973-JAN-1",
        "13",
        "1974-JAN-1",
        "14",
        "1975-JAN-1",
        "15",
        "1976-JAN-1",
        "16",
        "1977-JAN-1",
        "17",
        "1978-JAN-1",
        "18",
        "1979-JAN-1",
        "19",
        "1980-JAN-1",
        "20",
        "1981-JUL-1",
        "21",
        "1982-JUL-1",
        "22",
        "1983-JUL-1",
        "23",
        "1985-JUL-1",
        "24",
        "1988-JAN-1",
        "25",
        "1990-JAN-1",
        "26",
        "1991-JAN-1",
        "27",
        "1992-JUL-1",
        "28",
        "1993-JUL-1",
        "29",
        "1994-JUL-1",
        "30",
        "1996-JAN-1",
        "31",
        "1997-JUL-1",
        "32",
        "1999-JAN-1",
        "33",
        "2006-JAN-1",
        "34",
        "2009-JAN-1",
        "35",
        "2012-JUL-1",
        "36",
        "2015-JUL-1",
        "37",
        "2017-JAN-1",
    ];
    let act = kernel
        .get_timestamp_array("DELTET/DELTA_AT")
        .expect("array should be present");
    assert_eq!(act, &exp);

    assert!(kernel.get_timestamp_array("DELTET/DELTA_T_A").is_none());
    assert!(kernel.get_double("DELTET/DELTA_AT").is_none());
    assert!(kernel.get_double_array("DELTET/DELTA_AT").is_none());
}
