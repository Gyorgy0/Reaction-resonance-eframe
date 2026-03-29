use egui::epaint::{Color32, Hsva, Rgba};
use std::cmp::Ordering;
use std::fmt::{Display, Formatter};

/// The method used for interpolating between two points
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum InterpolationMethod {
    /// Use the nearest value to the left of the sample point. If there is no key point to the left
    /// of the sample, use the nearest point on the _right_ instead.
    Constant,
    /// Linearly interpolate between the two stops to the left and right of the sample. If the sample
    /// is outside the range of the stops, use the value of the single nearest stop.
    Linear,
}

impl Display for InterpolationMethod {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Linear => "linear",
                Self::Constant => "constant",
            }
        )
    }
}

/// A ColorInterpolator can arbitrarily sample a gradient.
pub struct ColorInterpolator {
    method: InterpolationMethod,
    keys: Vec<(f32, Rgba)>,
}

impl ColorInterpolator {
    fn new(
        keys: impl IntoIterator<Item = (f32, impl Into<Rgba>)>,
        method: InterpolationMethod,
    ) -> Self {
        let keys: Vec<_> = keys.into_iter().map(|(k, v)| (k, v.into())).collect();
        let mut result = Self { keys, method };
        result.sort();
        result
    }

    fn sort(&mut self) {
        self.keys
            .sort_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap());
    }

    /// Find the insertion point for x to maintain order
    fn bisect(&self, x: f32) -> Option<usize> {
        let mut lo = 0;
        let mut hi = self.keys.len();
        while lo < hi {
            let mid = (lo + hi) / 2;
            match self.keys[mid].0.partial_cmp(&x)? {
                Ordering::Less => lo = mid + 1,
                Ordering::Equal => lo = mid + 1,
                Ordering::Greater => hi = mid,
            }
        }

        Some(lo)
    }

    /// Sample the gradient at the given position.
    ///
    /// Returns `None` if the gradient is empty.
    pub fn sample_at(&self, x: f32) -> Option<Rgba> {
        Some(match self.method {
            InterpolationMethod::Constant => {
                let insertion_point = self.bisect(x)?;
                match insertion_point {
                    0 => self.keys.first()?.1,
                    n => self.keys.get(n - 1)?.1,
                }
            }
            InterpolationMethod::Linear => {
                let insertion_point = self.bisect(x)?;
                match insertion_point {
                    0 => self.keys.first()?.1,
                    n if n == self.keys.len() => self.keys.last()?.1,
                    n => {
                        let (t0, c0) = *self.keys.get(n - 1)?;
                        let (t1, c1) = *self.keys.get(n)?;

                        c0 + (c1 + c0 * -1.0_f32) * ((x - t0) / (t1 - t0))
                    }
                }
            }
        })
    }
}

/// A ColorInterpolator can arbitrarily sample a gradient.
pub struct ValueInterpolator {
    method: InterpolationMethod,
    keys: Vec<(usize, f32)>,
}

impl ValueInterpolator {
    fn new(keys: impl IntoIterator<Item = (usize, f32)>, method: InterpolationMethod) -> Self {
        let keys: Vec<_> = keys.into_iter().map(|(k, v)| (k, v.into())).collect();
        let mut result = Self { keys, method };
        result.sort();
        result
    }

    fn sort(&mut self) {
        self.keys
            .sort_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap());
    }

    /// Find the insertion point for x to maintain order
    fn bisect(&self, x: usize) -> Option<usize> {
        let mut lo = 0;
        let mut hi = self.keys.len();
        while lo < hi {
            let mid = (lo + hi) / 2;
            match self.keys[mid].0.partial_cmp(&x)? {
                Ordering::Less => lo = mid + 1,
                Ordering::Equal => lo = mid + 1,
                Ordering::Greater => hi = mid,
            }
        }

        Some(lo)
    }

    /// Sample the gradient at the given position.
    ///
    /// Returns `None` if the gradient is empty.
    pub fn sample_at(&self, x: usize) -> Option<usize> {
        Some(match self.method {
            InterpolationMethod::Constant => {
                let insertion_point = self.bisect(x)?;
                match insertion_point {
                    0 => self.keys.first()?.0,
                    n => self.keys.get(n - 1)?.0,
                }
            }
            InterpolationMethod::Linear => {
                let insertion_point = self.bisect(x)?;
                match insertion_point {
                    0 => self.keys.first()?.0,
                    n if n == self.keys.len() => self.keys.last()?.0,
                    n => {
                        let (t0, c0) = *self.keys.get(n - 1)?;
                        let (t1, c1) = *self.keys.get(n)?;

                        c0 as usize + (c1 + c0 * -1.0_f32) as usize * ((x - t0) / (t1 - t0))
                    }
                }
            }
        })
    }
}

fn argsort_by<T, F>(data: &[T], mut f: F) -> Vec<usize>
where
    F: FnMut(T, T) -> Ordering,
    T: Copy,
{
    let mut indices = (0..data.len()).collect::<Vec<_>>();
    indices.sort_by(|&a, &b| f(data[a], data[b]));
    indices
}

/// A color gradient, that will be interpolated between a number of fixed points, a.k.a. _stops_.
pub struct Gradient {
    pub stops: Vec<(f32, Hsva)>,
    pub interpolation_method: InterpolationMethod,
}

impl Gradient {
    /// Create a new gradient from an iterator over key colors.
    pub fn new(
        interpolation_method: InterpolationMethod,
        stops: impl IntoIterator<Item = (f32, impl Into<Hsva>)>,
    ) -> Self {
        Self {
            interpolation_method,
            stops: stops.into_iter().map(|(k, v)| (k, v.into())).collect(),
        }
    }

    /// Create a [ColorInterpolator] to evaluate the gradient at any point.
    pub fn interpolator(&self) -> ColorInterpolator {
        ColorInterpolator::new(self.stops.iter().copied(), self.interpolation_method)
    }

    /// Create a [ColorInterpolator] that discards the alpha component of the color gradient and
    /// always produces an opaque color.
    pub fn interpolator_opaque(&self) -> ColorInterpolator {
        ColorInterpolator::new(
            self.stops.iter().map(|(t, c)| (*t, c.to_opaque())),
            self.interpolation_method,
        )
    }

    /// Produce a list of the indices of the gradient's stops that would place them in order.
    ///
    /// Use this to prepare for the upcoming reordering of the stops by [sort()](Gradient::sort).
    pub fn argsort(&self) -> Vec<usize> {
        argsort_by(&self.stops, |(a, _), (b, _)| a.partial_cmp(&b).unwrap())
    }

    /// Sort the gradient's stops by ascending position.
    pub fn sort(&mut self) {
        self.stops
            .sort_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap())
    }

    /// Return a vector of the gradient's color sampled on linearly spaced points between 0 and 1.
    ///
    /// The first and last samples correspond to the gradient's value at 0.0 and 1.0, respectively.
    ///
    /// This is useful for generating a texture.
    ///
    /// # Panics
    ///
    /// Will panic if the provided size `n` is smaller or equal to 1, or if the gradient is empty.
    pub fn linear_eval(&self, n: usize, opaque: bool) -> Vec<Color32> {
        let interp = match opaque {
            false => self.interpolator(),
            true => self.interpolator_opaque(),
        };
        (0..n)
            .map(|idx| (idx as f32) / (n - 1) as f32)
            .map(|t| interp.sample_at(t).unwrap().into())
            .collect()
    }
}

impl Default for Gradient {
    fn default() -> Self {
        Self {
            stops: vec![(0., Color32::BLACK.into()), (1., Color32::WHITE.into())],
            interpolation_method: InterpolationMethod::Linear,
        }
    }
}

/// A color gradient, that will be interpolated between a number of fixed points, a.k.a. _stops_.
pub struct ValueGradient {
    pub stops: Vec<(usize, f32)>,
    pub interpolation_method: InterpolationMethod,
}

impl ValueGradient {
    /// Create a new gradient from an iterator over key colors.
    pub fn new(
        interpolation_method: InterpolationMethod,
        stops: impl IntoIterator<Item = (usize, f32)>,
    ) -> Self {
        Self {
            interpolation_method,
            stops: stops.into_iter().map(|(k, v)| (k, v.into())).collect(),
        }
    }

    /// Create a [ValueInterpolator] to evaluate the gradient at any point.
    pub fn interpolator(&self) -> ValueInterpolator {
        ValueInterpolator::new(self.stops.iter().copied(), self.interpolation_method)
    }

    /// Create a [ValueInterpolator] that discards the alpha component of the color gradient and
    /// always produces an opaque color.
    pub fn interpolator_opaque(&self) -> ValueInterpolator {
        ValueInterpolator::new(
            self.stops.iter().map(|(t, c)| (*t, *c)),
            self.interpolation_method,
        )
    }

    /// Produce a list of the indices of the gradient's stops that would place them in order.
    ///
    /// Use this to prepare for the upcoming reordering of the stops by [sort()](Gradient::sort).
    pub fn argsort(&self) -> Vec<usize> {
        argsort_by(&self.stops, |(a, _), (b, _)| a.partial_cmp(&b).unwrap())
    }

    /// Sort the gradient's stops by ascending position.
    pub fn sort(&mut self) {
        self.stops
            .sort_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap())
    }

    /// Return a vector of the gradient's color sampled on linearly spaced points between 0 and 1.
    ///
    /// The first and last samples correspond to the gradient's value at 0.0 and 1.0, respectively.
    ///
    /// This is useful for generating a texture.
    ///
    /// # Panics
    ///
    /// Will panic if the provided size `n` is smaller or equal to 1, or if the gradient is empty.
    pub fn linear_eval(&self, n: usize, opaque: bool) -> Vec<f32> {
        let interp = match opaque {
            false => self.interpolator(),
            true => self.interpolator_opaque(),
        };
        (0..n)
            .map(|idx| (idx as usize) / (n - 1) as usize)
            .map(|t| interp.sample_at(t).unwrap() as f32)
            .collect()
    }
}
