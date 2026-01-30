use crate::{
    curves::{gray, hairyonion, hcurve, hilbert, onion, scan, zorder},
    error,
    spacecurve::SpaceCurve,
    spec::GridSpec,
    types::{Coord, DefaultCoord, DefaultIndex, Index},
};

/// Boxed curve constructor used by the registry.
type CurveCtor<C, I> =
    fn(&GridSpec<C, I>) -> error::Result<Box<dyn SpaceCurve<Coord = C, Index = I>>>;

/// Metadata and constructor for a curve type.
pub struct CurveEntry<C: Coord, I: Index> {
    /// Canonical, lowercase key (as accepted by CLI/APIs).
    pub key: &'static str,
    /// Human-friendly display name.
    pub display: &'static str,
    /// Human-friendly constraints summary suitable for help text.
    pub constraints: String,
    /// Whether this curve is experimental and should be hidden in stable UIs.
    pub experimental: bool,
    /// Build a validated grid specification for this curve.
    pub build_spec: fn(u32, C) -> error::Result<GridSpec<C, I>>,
    /// Construct the curve given a validated grid specification.
    pub ctor: CurveCtor<C, I>,
}

// --- Per-curve validators -----------------------------------------------------

/// Hilbert pre-validation aligned with constructor invariants.
fn v_hilbert<C: Coord, I: Index>(dim: u32, size: C) -> error::Result<GridSpec<C, I>> {
    let spec = GridSpec::<C, I>::power_of_two(dim, size)?;
    spec.require_index_bits_lt(I::BITS)?;
    Ok(spec)
}

/// H-curve pre-validation aligned with constructor invariants.
fn v_hcurve<C: Coord, I: Index>(dim: u32, size: C) -> error::Result<GridSpec<C, I>> {
    if dim < 2 {
        return Err(error::Error::Shape("dimension must be >= 2".to_string()));
    }
    if dim >= 32 {
        return Err(error::Error::Shape("dimension must be < 32".to_string()));
    }
    let spec = GridSpec::<C, I>::power_of_two(dim, size)?;
    let total_bits = (spec.order().unwrap() as u128) * (dim as u128);
    if total_bits >= I::BITS as u128 {
        return Err(error::Error::Size(format!(
            "Curve size exceeds index limits (D*O must be < {})",
            I::BITS
        )));
    }
    Ok(spec)
}

/// Z-order (Morton) pre-validation aligned with constructor invariants.
fn v_zorder<C: Coord, I: Index>(dim: u32, size: C) -> error::Result<GridSpec<C, I>> {
    let spec = GridSpec::<C, I>::power_of_two(dim, size)?;
    spec.require_index_bits_lt(I::BITS)?;
    Ok(spec)
}

/// Onion pre-validation: generic shape/length checks.
fn v_onion<C: Coord, I: Index>(dim: u32, size: C) -> error::Result<GridSpec<C, I>> {
    GridSpec::<C, I>::new(dim, size)
}

/// Hairy Onion pre-validation: generic shape/length checks.
fn v_hairyonion<C: Coord, I: Index>(dim: u32, size: C) -> error::Result<GridSpec<C, I>> {
    GridSpec::<C, I>::new(dim, size)
}

/// Scan pre-validation: generic shape/length checks.
fn v_scan<C: Coord, I: Index>(dim: u32, size: C) -> error::Result<GridSpec<C, I>> {
    GridSpec::<C, I>::new(dim, size)
}

/// Gray pre-validation: generic shape/length checks.
fn v_gray<C: Coord, I: Index>(dim: u32, size: C) -> error::Result<GridSpec<C, I>> {
    let spec = GridSpec::<C, I>::power_of_two(dim, size)?;
    spec.require_index_bits_lt(I::BITS)?;
    Ok(spec)
}

// --- Per-curve constructors (boxed trait objects) ----------------------------

/// Construct a boxed Hilbert instance.
fn c_hilbert<C: Coord, I: Index>(
    spec: &GridSpec<C, I>,
) -> error::Result<Box<dyn SpaceCurve<Coord = C, Index = I>>> {
    Ok(Box::new(hilbert::Hilbert::<C, I>::from_dimensions(
        spec.dimension(),
        spec.size(),
    )?))
}

/// Construct a boxed H-curve instance.
fn c_hcurve<C: Coord, I: Index>(
    spec: &GridSpec<C, I>,
) -> error::Result<Box<dyn SpaceCurve<Coord = C, Index = I>>> {
    Ok(Box::new(hcurve::HCurve::<C, I>::from_dimensions(
        spec.dimension(),
        spec.size(),
    )?))
}

/// Construct a boxed Z-order instance.
fn c_zorder<C: Coord, I: Index>(
    spec: &GridSpec<C, I>,
) -> error::Result<Box<dyn SpaceCurve<Coord = C, Index = I>>> {
    Ok(Box::new(zorder::ZOrder::<C, I>::from_dimensions(
        spec.dimension(),
        spec.size(),
    )?))
}

/// Construct a boxed Onion instance.
fn c_onion<C: Coord, I: Index>(
    spec: &GridSpec<C, I>,
) -> error::Result<Box<dyn SpaceCurve<Coord = C, Index = I>>> {
    Ok(Box::new(onion::OnionCurve::<C, I>::new(
        spec.dimension(),
        spec.size(),
    )?))
}

/// Construct a boxed Hairy Onion instance.
fn c_hairyonion<C: Coord, I: Index>(
    spec: &GridSpec<C, I>,
) -> error::Result<Box<dyn SpaceCurve<Coord = C, Index = I>>> {
    Ok(Box::new(hairyonion::HairyOnionCurve::<C, I>::new(
        spec.dimension(),
        spec.size(),
    )?))
}

/// Construct a boxed Scan instance.
fn c_scan<C: Coord, I: Index>(
    spec: &GridSpec<C, I>,
) -> error::Result<Box<dyn SpaceCurve<Coord = C, Index = I>>> {
    Ok(Box::new(scan::Scan::<C, I>::from_dimensions(
        spec.dimension(),
        spec.size(),
    )?))
}

/// Construct a boxed Gray instance.
fn c_gray<C: Coord, I: Index>(
    spec: &GridSpec<C, I>,
) -> error::Result<Box<dyn SpaceCurve<Coord = C, Index = I>>> {
    Ok(Box::new(gray::Gray::<C, I>::from_dimensions(
        spec.dimension(),
        spec.size(),
    )?))
}

/// Public list of curve keys accepted by the library and CLI.
pub const CURVE_NAMES: &[&str] = &[
    "hilbert",
    "scan",
    "zorder",
    "hcurve",
    "onion",
    "hairyonion",
    "gray",
];

/// Build the registry entries for the requested numeric widths.
fn registry<C: Coord, I: Index>() -> Vec<CurveEntry<C, I>> {
    let index_bits = I::BITS;
    vec![
        CurveEntry {
            key: "hilbert",
            display: "Hilbert",
            constraints: format!(
                "size=2^order; order*dimension < {} (index bits)",
                index_bits
            ),
            experimental: false,
            build_spec: v_hilbert::<C, I>,
            ctor: c_hilbert::<C, I>,
        },
        CurveEntry {
            key: "scan",
            display: "Scan",
            constraints: "any size>=1; any dimension>=1".to_string(),
            experimental: false,
            build_spec: v_scan::<C, I>,
            ctor: c_scan::<C, I>,
        },
        CurveEntry {
            key: "zorder",
            display: "Z-order (Morton)",
            constraints: format!(
                "size=2^bitwidth; bitwidth*dimension < {} (index bits)",
                index_bits
            ),
            experimental: false,
            build_spec: v_zorder::<C, I>,
            ctor: c_zorder::<C, I>,
        },
        CurveEntry {
            key: "hcurve",
            display: "H-curve",
            constraints: format!(
                "dimension>=2; dimension<32; size=2^order; order*dimension < {}",
                index_bits
            ),
            experimental: false,
            build_spec: v_hcurve::<C, I>,
            ctor: c_hcurve::<C, I>,
        },
        CurveEntry {
            key: "onion",
            display: "Onion",
            constraints: format!(
                "any size>=1; any dimension>=1; length=size^dimension fits {}-bit index",
                index_bits
            ),
            experimental: false,
            build_spec: v_onion::<C, I>,
            ctor: c_onion::<C, I>,
        },
        CurveEntry {
            key: "hairyonion",
            display: "Hairy Onion",
            constraints: format!(
                "any size>=1; any dimension>=1; length=size^dimension fits {}-bit index",
                index_bits
            ),
            experimental: true,
            build_spec: v_hairyonion::<C, I>,
            ctor: c_hairyonion::<C, I>,
        },
        CurveEntry {
            key: "gray",
            display: "Gray (BRGC)",
            constraints: format!(
                "size=2^bitwidth; bitwidth*dimension < {} (index bits)",
                index_bits
            ),
            experimental: false,
            build_spec: v_gray::<C, I>,
            ctor: c_gray::<C, I>,
        },
    ]
}

/// Return curve keys, optionally filtering out experimental entries.
pub fn curve_names(include_experimental: bool) -> Vec<&'static str> {
    registry::<DefaultCoord, DefaultIndex>()
        .into_iter()
        .filter(|entry| include_experimental || !entry.experimental)
        .map(|entry| entry.key)
        .collect()
}

/// Look up a registry entry by key (case-sensitive).
pub fn find<C: Coord, I: Index>(key: &str) -> Option<CurveEntry<C, I>> {
    registry::<C, I>().into_iter().find(|e| e.key == key)
}

/// Validate a curve specification using the registry without constructing it.
pub fn validate<C: Coord, I: Index>(key: &str, dimension: u32, size: C) -> error::Result<()> {
    match find::<C, I>(key) {
        Some(entry) => {
            (entry.build_spec)(dimension, size)?;
            Ok(())
        }
        None => Err(error::Error::Unknown(format!("unknown pattern: \"{key}\""))),
    }
}

/// Construct a curve by key after validating via the registry.
pub fn construct<C: Coord, I: Index>(
    key: &str,
    dimension: u32,
    size: C,
) -> error::Result<Box<dyn SpaceCurve<Coord = C, Index = I> + 'static>> {
    match find::<C, I>(key) {
        Some(entry) => {
            let spec = (entry.build_spec)(dimension, size)?;
            (entry.ctor)(&spec)
        }
        None => Err(error::Error::Unknown(format!("unknown pattern: \"{key}\""))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_consistency() {
        let mut registry_keys: Vec<&str> = registry::<DefaultCoord, DefaultIndex>()
            .iter()
            .map(|e| e.key)
            .collect();
        let mut names_list: Vec<&str> = CURVE_NAMES.to_vec();

        registry_keys.sort();
        names_list.sort();

        assert_eq!(
            registry_keys, names_list,
            "registry keys and CURVE_NAMES must be identical"
        );
    }
}
