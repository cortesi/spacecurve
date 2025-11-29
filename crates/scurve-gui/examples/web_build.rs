//! Minimal example crate to ensure the web entrypoint compiles on wasm32.
// Tiny example to ensure the web target compiles.
// On wasm32, we include the real web entrypoint (src/web.rs), which defines
// the wasm_bindgen start function and a `main`, guaranteeing that the same
// code path used for the web binary is built. On native targets, this example
// is a no-op so it can compile everywhere.

#![allow(dead_code)]

#[cfg(target_arch = "wasm32")]
include!("../src/web.rs");

#[cfg(not(target_arch = "wasm32"))]
fn main() {}
