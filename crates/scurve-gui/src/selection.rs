use spacecurve::{DefaultIndex, curve_from_name, registry};

/// Shared cache and selection state for 2D/3D curve panes.
#[derive(Clone)]
pub struct CurveSelection<const D: usize> {
    /// The selected curve name.
    pub name: String,
    /// The side length of the grid per axis.
    pub size: u32,
    /// Current offset for the animated snake overlay, in segments.
    pub snake_offset: f32,
    /// Whether the info pane for this curve is open.
    pub info_open: bool,
    /// Cache key: last curve name used to generate `cached_points`.
    cached_name: String,
    /// Cache key: last grid size used to generate `cached_points`.
    cached_size: u32,
    /// Cached integer points for the currently selected curve and size.
    cached_points: Vec<[u32; D]>,
    /// Cached curve length for the currently selected curve and size.
    cached_length: Option<DefaultIndex>,
}

impl<const D: usize> Default for CurveSelection<D> {
    fn default() -> Self {
        let default_name = registry::curve_names(false)
            .first()
            .copied()
            .unwrap_or(registry::CURVE_NAMES[0]);

        Self::with_name(default_name)
    }
}

impl<const D: usize> CurveSelection<D> {
    /// Build a selection with a specific initial curve name.
    pub fn with_name(name: &str) -> Self {
        Self {
            name: name.to_string(),
            size: if D == 2 { 64 } else { 8 },
            snake_offset: 0.0,
            info_open: false,
            cached_name: String::new(),
            cached_size: 0,
            cached_points: Vec::new(),
            cached_length: None,
        }
    }

    /// Reset cached data when the selected curve or size changes.
    fn invalidate_if_changed(&mut self) {
        if self.cached_name != self.name || self.cached_size != self.size {
            self.cached_points.clear();
            self.cached_length = None;
        }
    }

    /// Ensure the cached curve length is available for the current selection.
    pub fn ensure_curve_length(&mut self) -> Option<DefaultIndex> {
        self.invalidate_if_changed();
        if let Some(len) = self.cached_length {
            return Some(len);
        }
        if !self.cached_points.is_empty() {
            let len = self.cached_points.len() as DefaultIndex;
            self.cached_length = Some(len);
            return Some(len);
        }
        match curve_from_name(&self.name, D as u32, self.size) {
            Ok(pattern) => {
                let len = pattern.length();
                self.cached_length = Some(len);
                self.cached_name = self.name.clone();
                self.cached_size = self.size;
                Some(len)
            }
            Err(_) => None,
        }
    }

    /// Ensure the cached points are computed for the current name and size.
    /// Returns a slice of cached points if successful.
    pub fn ensure_cached_points(&mut self) -> Option<&[[u32; D]]> {
        self.invalidate_if_changed();
        if self.cached_name != self.name
            || self.cached_size != self.size
            || self.cached_points.is_empty()
        {
            if let Ok(pattern) = curve_from_name(&self.name, D as u32, self.size) {
                let length = pattern.length();
                let length_usize = usize::try_from(length).ok()?;
                let mut pts = Vec::with_capacity(length_usize);
                for i in 0..length {
                    let p = pattern.point(i);
                    let mut arr = [0u32; D];
                    for d in 0..D {
                        arr[d] = p[d];
                    }
                    pts.push(arr);
                }
                self.cached_points = pts;
                self.cached_name = self.name.clone();
                self.cached_size = self.size;
                self.cached_length = Some(length);
            } else {
                return None;
            }
        }
        Some(&self.cached_points)
    }
}

/// 2D selection state.
pub type SelectedCurve = CurveSelection<2>;
/// 3D selection state.
pub type Selected3DCurve = CurveSelection<3>;
