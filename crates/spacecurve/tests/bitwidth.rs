#![allow(missing_docs, clippy::tests_outside_test_module)]

use spacecurve::curve_from_name_typed;

#[test]
fn scan_4d_1024_u64_index_length() {
    let curve =
        curve_from_name_typed::<u32, u64>("scan", 4, 1024).expect("scan curve with u64 index");
    assert_eq!(curve.length(), 1_099_511_627_776u64);
}

#[test]
fn scan_4d_1024_u128_index_length() {
    let curve =
        curve_from_name_typed::<u32, u128>("scan", 4, 1024).expect("scan curve with u128 index");
    assert_eq!(curve.length(), 1_099_511_627_776u128);
}
