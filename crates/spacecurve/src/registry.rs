use crate::{
    curves::{gray, hairyonion, hcurve, hilbert, onion, scan, zorder},
    error,
    spacecurve::SpaceCurve,
    spec::GridSpec,
};

/// Metadata and constructor for a curve type.
pub struct CurveEntry {
    /// Canonical, lowercase key (as accepted by CLI/APIs).
    pub key: &'static str,
    /// Human-friendly display name.
    pub display: &'static str,
    /// Human-friendly constraints summary suitable for help text.
    pub constraints: &'static str,
    /// Whether this curve is experimental and should be hidden in stable UIs.
    pub experimental: bool,
    /// Build a validated grid specification for this curve.
    pub build_spec: fn(u32, u32) -> error::Result<GridSpec>,
    /// Construct the curve given a validated grid specification.
    pub ctor: fn(&GridSpec) -> error::Result<Box<dyn SpaceCurve + 'static>>,
}

// --- Per-curve validators -----------------------------------------------------

/// Hilbert pre-validation aligned with constructor invariants.
fn v_hilbert(dim: u32, size: u32) -> error::Result<GridSpec> {
    let spec = GridSpec::power_of_two(dim, size)?;
    let total_bits = (spec.order().unwrap() as u64) * (dim as u64);
    if total_bits >= 32 {
        return Err(error::Error::Size(
            "Hilbert requires order * dimension < 32 for u32 indices".to_string(),
        ));
    }
    Ok(spec)
}

/// H-curve pre-validation aligned with constructor invariants.
fn v_hcurve(dim: u32, size: u32) -> error::Result<GridSpec> {
    if dim < 2 {
        return Err(error::Error::Shape("dimension must be >= 2".to_string()));
    }
    let spec = GridSpec::power_of_two(dim, size)?;
    if dim >= 32 {
        return Err(error::Error::Shape("dimension must be < 32".to_string()));
    }
    if (spec.order().unwrap() as u64) * (dim as u64) >= 32 {
        return Err(error::Error::Size(
            "Curve size exceeds u32 limits (D*O must be < 32)".to_string(),
        ));
    }
    Ok(spec)
}

/// Z-order (Morton) pre-validation aligned with constructor invariants.
fn v_zorder(dim: u32, size: u32) -> error::Result<GridSpec> {
    let spec = GridSpec::power_of_two(dim, size)?;
    spec.require_index_bits_lt(32)?;
    Ok(spec)
}

/// Onion pre-validation: generic shape/length checks.
fn v_onion(dim: u32, size: u32) -> error::Result<GridSpec> {
    GridSpec::new(dim, size)
}

/// Hairy Onion pre-validation: generic shape/length checks.
fn v_hairyonion(dim: u32, size: u32) -> error::Result<GridSpec> {
    GridSpec::new(dim, size)
}

/// Scan pre-validation: generic shape/length checks.
fn v_scan(dim: u32, size: u32) -> error::Result<GridSpec> {
    GridSpec::new(dim, size)
}

/// Gray pre-validation: generic shape/length checks.
fn v_gray(dim: u32, size: u32) -> error::Result<GridSpec> {
    let spec = GridSpec::power_of_two(dim, size)?;
    if (spec.bits_per_axis().unwrap() as u64) * (dim as u64) >= 32 {
        return Err(error::Error::Size(
            "Gray requires bitwidth * dimension < 32 for u32 indices".to_string(),
        ));
    }
    Ok(spec)
}

// --- Per-curve constructors (boxed trait objects) ----------------------------

/// Construct a boxed Hilbert instance.
fn c_hilbert(spec: &GridSpec) -> error::Result<Box<dyn SpaceCurve + 'static>> {
    Ok(Box::new(hilbert::Hilbert::from_dimensions(
        spec.dimension(),
        spec.size(),
    )?))
}
/// Construct a boxed H-curve instance.
fn c_hcurve(spec: &GridSpec) -> error::Result<Box<dyn SpaceCurve + 'static>> {
    Ok(Box::new(hcurve::HCurve::from_dimensions(
        spec.dimension(),
        spec.size(),
    )?))
}
/// Construct a boxed Z-order instance.
fn c_zorder(spec: &GridSpec) -> error::Result<Box<dyn SpaceCurve + 'static>> {
    Ok(Box::new(zorder::ZOrder::from_dimensions(
        spec.dimension(),
        spec.size(),
    )?))
}
/// Construct a boxed Onion instance.
fn c_onion(spec: &GridSpec) -> error::Result<Box<dyn SpaceCurve + 'static>> {
    Ok(Box::new(onion::OnionCurve::new(
        spec.dimension(),
        spec.size(),
    )?))
}
/// Construct a boxed Hairy Onion instance.
fn c_hairyonion(spec: &GridSpec) -> error::Result<Box<dyn SpaceCurve + 'static>> {
    Ok(Box::new(hairyonion::HairyOnionCurve::new(
        spec.dimension(),
        spec.size(),
    )?))
}
/// Construct a boxed Scan instance.
fn c_scan(spec: &GridSpec) -> error::Result<Box<dyn SpaceCurve + 'static>> {
    Ok(Box::new(scan::Scan::from_dimensions(
        spec.dimension(),
        spec.size(),
    )?))
}
/// Construct a boxed Gray instance.
fn c_gray(spec: &GridSpec) -> error::Result<Box<dyn SpaceCurve + 'static>> {
    Ok(Box::new(gray::Gray::from_dimensions(
        spec.dimension(),
        spec.size(),
    )?))
}

/// Generate the registry table and the ordered list of curve keys from one
/// token list to avoid drift between the two.
macro_rules! define_registry {
    ( $(
        {
            $key:literal,
            $display:literal,
            $constraints:literal,
            $experimental:expr,
            $validate:ident,
            $ctor:ident
        }
    ),+ $(,)? ) => {
        /// Public list of curve keys accepted by the library and CLI.
        pub const CURVE_NAMES: &[&str] = &[ $( $key ),+ ];

        /// Static registry map. Keys must match `CURVE_NAMES` order.
        pub static REGISTRY: &[CurveEntry] = &[
            $(
                CurveEntry {
                    key: $key,
                    display: $display,
                    constraints: $constraints,
                    experimental: $experimental,
                    build_spec: $validate,
                    ctor: $ctor,
                },
            )+
        ];
    };
}

define_registry! {
    { "hilbert", "Hilbert", "size=2^order; order*dimension < 32 (u32 indices)", false, v_hilbert, c_hilbert },
    { "scan", "Scan", "any size>=1; any dimension>=1", false, v_scan, c_scan },
    { "zorder", "Z-order (Morton)", "size=2^bitwidth; bitwidth*dimension < 32 (u32 indices)", false, v_zorder, c_zorder },
    { "hcurve", "H-curve", "dimension>=2; size=2^order; order*dimension < 32", false, v_hcurve, c_hcurve },
    { "onion", "Onion", "any size>=1; any dimension>=1; length=size^dimension fits u32", false, v_onion, c_onion },
    { "hairyonion", "Hairy Onion", "any size>=1; any dimension>=1; length=size^dimension fits u32", true, v_hairyonion, c_hairyonion },
    { "gray", "Gray (BRGC)", "size=2^bitwidth; bitwidth*dimension < 32 (u32 indices)", false, v_gray, c_gray },
}

/// Return curve keys, optionally filtering out experimental entries.
pub fn curve_names(include_experimental: bool) -> Vec<&'static str> {
    REGISTRY
        .iter()
        .filter(|entry| include_experimental || !entry.experimental)
        .map(|entry| entry.key)
        .collect()
}

/// Look up a registry entry by key (case-sensitive).
pub fn find(key: &str) -> Option<&'static CurveEntry> {
    REGISTRY.iter().find(|e| e.key == key)
}

/// Validate a curve specification using the registry without constructing it.
pub fn validate(key: &str, dimension: u32, size: u32) -> error::Result<()> {
    match find(key) {
        Some(entry) => {
            (entry.build_spec)(dimension, size)?;
            Ok(())
        }
        None => Err(error::Error::Unknown(format!("unknown pattern: \"{key}\""))),
    }
}

/// Construct a curve by key after validating via the registry.
pub fn construct(
    key: &str,
    dimension: u32,
    size: u32,
) -> error::Result<Box<dyn SpaceCurve + 'static>> {
    match find(key) {
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
        let mut registry_keys: Vec<&str> = REGISTRY.iter().map(|e| e.key).collect();
        let mut names_list: Vec<&str> = CURVE_NAMES.to_vec();

        registry_keys.sort();
        names_list.sort();

        assert_eq!(
            registry_keys, names_list,
            "REGISTRY keys and CURVE_NAMES must be identical"
        );

        // Also ensure they are in the same order in the source arrays if that matters
        // (though the sort above handles content equality).
        // If precise order matching is required by the consuming code:
        for (i, name) in CURVE_NAMES.iter().enumerate() {
            assert_eq!(
                REGISTRY[i].key, *name,
                "REGISTRY and CURVE_NAMES order mismatch at index {}",
                i
            );
        }
    }
}
