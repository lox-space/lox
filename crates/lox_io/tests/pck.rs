use lox_io::Kernel;

#[test]
fn test_pck() {
    let pck = include_str!("pck00011.tpc");
    let kernel = Kernel::parse(pck).expect("file should be parsable");
    assert_eq!(kernel.type_id(), "PCK");

    let exp = vec![286.13, 0., 0.];
    let act = kernel
        .get_double_array("BODY10_POLE_RA")
        .expect("array should be present");
    assert_eq!(act, &exp);

    let exp = vec![2.40, 1.55, 1.20];
    let act = kernel
        .get_double_array("BODY1000012_RADII")
        .expect("array should be present");
    assert_eq!(act, &exp);

    let act = kernel
        .get_double("BODY4_MAX_PHASE_DEGREE")
        .expect("value should be present");
    assert_eq!(act, 2.0);
}
