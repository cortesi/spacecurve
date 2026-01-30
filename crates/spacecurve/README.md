spacecurve
==========

A Rust library for N-dimensional space-filling curves and spatial indexing.

## Features

*   **Supported Curves:**
    *   **Hilbert** (2D optimized, N-D generic)
    *   **Z-order / Morton** (optimized bit-interleaving)
    *   **Gray Code** (Binary Reflected)
    *   **H-curve**
    *   **Scan** (Boustrophedon)
    *   **Onion** / **Hairy Onion** (Recursive layer-based)
*   **High Performance:** Uses `SmallVec` to avoid heap allocations for common 2D/3D points, and optimized SWAR algorithms for bit manipulation.
*   **Generic:** Supports N-dimensional mappings where applicable.

## Numeric widths

Curves are generic over coordinate and index widths. The default helpers use
`DefaultCoord` (`u32`) and `DefaultIndex` (`u64`). To opt into other widths,
call `curve_from_name_typed::<C, I>` or construct a concrete curve type
directly; there are no runtime width switches.

## Usage

```rust
use std::error::Error;

use spacecurve::{curve_from_name, curve_from_name_typed, SpaceCurve};

fn main() -> Result<(), Box<dyn Error>> {
    // Create a 2D Hilbert curve of order 3 (8x8 grid)
    let curve = curve_from_name("hilbert", 2, 8)?;

    let point = curve.point(10);
    println!("Point at index 10: {:?}", point);

    let index = curve.index(&point);
    assert_eq!(index, 10);

    // Opt into a wider index type for larger grids:
    let wide = curve_from_name_typed::<u32, u128>("scan", 4, 1024)?;
    println!("4D scan length: {}", wide.length());

    Ok(())
}
```

More usage is available in `examples/hilbert.rs`.
