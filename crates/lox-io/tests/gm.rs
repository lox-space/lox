// SPDX-FileCopyrightText: 2023 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use lox_io::spice::Kernel;
use lox_test_utils::read_data_file;

#[test]
fn test_gm() {
    let gm = read_data_file("spice/gm_de440.tpc");
    let kernel = Kernel::from_string(&gm).expect("file should be parsable");
    assert_eq!(kernel.type_id(), "PCK");

    assert!(!kernel.keys().is_empty());

    let exp = vec![2.203_186_855_140_000_3e4];
    let act = kernel
        .get_double_array("BODY1_GM")
        .expect("array should be present");
    assert_eq!(act, &exp);

    let exp = vec![0.0];
    let act = kernel
        .get_double_array("BODY153092511_GM")
        .expect("array should be present");
    assert_eq!(act, &exp);

    assert!(kernel.get_double("foo").is_none());
    assert!(kernel.get_double_array("foo").is_none());
}
