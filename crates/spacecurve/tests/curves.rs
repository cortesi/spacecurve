//! Integration tests checking reflection and continuity properties.
#[cfg(test)]
mod tests {
    use spacecurve::{
        error, onion::OnionCurve, pattern_from_name, point::Point, spacecurve::SpaceCurve,
    };

    fn pattern_reflects(pattern_name: &str, p: &dyn SpaceCurve) {
        for off in 0..p.length() {
            let pt = p.point(off);
            let off2 = p.index(&pt);
            assert_eq!(
                off2, off,
                "Pattern {pattern_name} does not reflect: {off} -> {pt:?} -> {off2}"
            );
        }
    }

    fn pattern_continuous(pattern_name: &str, p: &dyn SpaceCurve) {
        for off in 1..p.length() {
            let pt1 = p.point(off);
            let pt2 = p.point(off - 1);
            assert_eq!(
                pt1.distance(&pt2),
                1.0,
                "Pattern {} is discontinuous at offset {}: distance between {:?} and {:?} is {}",
                pattern_name,
                off - 1,
                pt2,
                pt1,
                pt1.distance(&pt2)
            );
        }
    }

    macro_rules! curve_tests {
        ($(($pattern:expr, $dims:expr, $size:expr, $reflection:expr, $continuous:expr)),* $(,)?) => {
            $(
                paste::paste! {
                    #[test]
                    fn [<$pattern _reflection_ $dims d_ $size>]() -> error::Result<()> {
                        if $reflection {
                            let curve = pattern_from_name($pattern, $dims, $size)?;
                            pattern_reflects(&format!("{}({},{})", $pattern, $dims, $size), curve.as_ref());
                        }
                        Ok(())
                    }

                    #[test]
                    fn [<$pattern _continuous_ $dims d_ $size>]() -> error::Result<()> {
                        if $continuous {
                            let curve = pattern_from_name($pattern, $dims, $size)?;
                            pattern_continuous(&format!("{}({},{})", $pattern, $dims, $size), curve.as_ref());
                        }
                        Ok(())
                    }
                }
            )*
        };
    }

    curve_tests! {
        ("hilbert", 2, 4, true, true),
        ("hilbert", 3, 4, true, true),
        ("hilbert", 4, 2, true, true),
        ("hcurve", 2, 4, true, true),
        // ("hcurve", 3, 4, true, true),
        // ("hcurve", 3, 8, true, true),
        ("hcurve", 4, 2, true, true),
        ("scan", 2, 4, true, true),
        ("scan", 3, 4, true, true),
        ("scan", 4, 2, true, true),
        ("zorder", 2, 4, true, false),
        ("zorder", 3, 4, true, false),
        ("zorder", 4, 2, true, false),
        ("onion", 2, 4, true, true),
        ("onion", 3, 4, true, false),
        ("onion", 4, 2, true, false),
        ("hairyonion", 2, 4, true, true),
        ("hairyonion", 3, 4, true, true),
        ("hairyonion", 4, 2, true, true),
        ("gray", 2, 4, true, false),
        ("gray", 3, 4, true, false),
        ("gray", 4, 2, true, false),
    }

    #[test]
    fn onion_3d_outer_faces_follow_plane_order() -> error::Result<()> {
        let face = OnionCurve::new(2, 5)?;
        let cube = OnionCurve::new(3, 5)?;

        for idx in 0..25u32 {
            let p3 = cube.point(idx);
            assert_eq!(p3[0], 0, "Index {} should be on the x=0 face", idx);

            let p2 = face.point(idx);
            assert_eq!(p2[0], p3[1]);
            assert_eq!(p2[1], p3[2]);

            let reconstructed = Point::new(vec![0, p2[0], p2[1]]);
            assert_eq!(cube.index(&reconstructed), idx);
        }
        Ok(())
    }

    #[test]
    fn onion_3d_initial_edge_sequence_matches_definition() -> error::Result<()> {
        let cube = OnionCurve::new(3, 5)?;

        // First two faces occupy indices [0, 50).
        assert_eq!(cube.point(0).as_slice(), &[0, 0, 0]);
        assert_eq!(cube.point(25).as_slice(), &[4, 0, 0]);

        // Following three indices should cover the S3(t) line along x with y=z=0.
        assert_eq!(cube.point(50).as_slice(), &[1, 0, 0]);
        assert_eq!(cube.point(51).as_slice(), &[2, 0, 0]);
        assert_eq!(cube.point(52).as_slice(), &[3, 0, 0]);

        // Confirm index lookups for the same coordinates.
        for (expected, coords) in [(50u32, [1, 0, 0]), (51, [2, 0, 0]), (52, [3, 0, 0])] {
            let pt = Point::new(coords.to_vec());
            assert_eq!(cube.index(&pt), expected);
        }

        Ok(())
    }
}
