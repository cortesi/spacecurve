#![allow(missing_docs, clippy::tests_outside_test_module)]

use std::process::Command;

use assert_cmd::prelude::*;

#[test]
#[allow(deprecated)]
fn list_curves_prints_expected_entries() {
    let mut cmd = Command::cargo_bin("scurve").expect("binary exists");
    cmd.arg("list-curves");
    let assert = cmd.assert().success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("hilbert"));
    assert!(stdout.contains("Z-order (Morton)"));
}
