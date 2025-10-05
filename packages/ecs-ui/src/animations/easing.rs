//! High-performance easing functions for smooth animations

/// High-performance easing functions with GPU-friendly implementations
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EasingFunction {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    EaseInQuad,
    EaseOutQuart,
    EaseInBack,
    EaseOutBack,
    EaseInOutBack,
    EaseInElastic,
    EaseOutElastic,
    EaseInOutElastic,
    EaseInBounce,
    EaseOutBounce,
    EaseInOutBounce,
}

impl EasingFunction {
    /// Apply easing function with optimized math operations
    #[inline]
    pub fn apply(self, t: f32) -> f32 {
        match self {
            Self::Linear => t,
            Self::EaseIn => t * t * t,
            Self::EaseOut => {
                let t = t - 1.0;
                1.0 + t * t * t
            },
            Self::EaseInOut => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    let t = 2.0 * t - 2.0;
                    1.0 + t * t * t * 0.5
                }
            },
            Self::EaseInQuad => t * t,
            Self::EaseOutQuart => {
                let t = t - 1.0;
                1.0 - t * t * t * t
            },
            Self::EaseInBack => {
                const C1: f32 = 1.70158;
                const C3: f32 = C1 + 1.0;
                C3 * t * t * t - C1 * t * t
            },
            Self::EaseOutBack => {
                const C1: f32 = 1.70158;
                const C3: f32 = C1 + 1.0;
                let t = t - 1.0;
                1.0 + C3 * t * t * t + C1 * t * t
            },
            Self::EaseInOutBack => {
                const C1: f32 = 1.70158;
                const C2: f32 = C1 * 1.525;
                if t < 0.5 {
                    let t = 2.0 * t;
                    0.5 * (t * t * ((C2 + 1.0) * t - C2))
                } else {
                    let t = 2.0 * t - 2.0;
                    0.5 * (t * t * ((C2 + 1.0) * t + C2) + 2.0)
                }
            },
            Self::EaseInElastic => {
                if t == 0.0 {
                    0.0
                } else if t == 1.0 {
                    1.0
                } else {
                    const C4: f32 = std::f32::consts::TAU / 3.0;
                    -(2.0_f32.powf(10.0 * t - 10.0)) * ((t * 10.0 - 10.75) * C4).sin()
                }
            },
            Self::EaseOutElastic => {
                if t == 0.0 {
                    0.0
                } else if t == 1.0 {
                    1.0
                } else {
                    const C4: f32 = std::f32::consts::TAU / 3.0;
                    2.0_f32.powf(-10.0 * t) * ((t * 10.0 - 0.75) * C4).sin() + 1.0
                }
            },
            Self::EaseInOutElastic => {
                if t == 0.0 {
                    0.0
                } else if t == 1.0 {
                    1.0
                } else {
                    const C5: f32 = std::f32::consts::TAU / 4.5;
                    if t < 0.5 {
                        -0.5 * 2.0_f32.powf(20.0 * t - 10.0) * ((20.0 * t - 11.125) * C5).sin()
                    } else {
                        0.5 * 2.0_f32.powf(-20.0 * t + 10.0) * ((20.0 * t - 11.125) * C5).sin()
                            + 1.0
                    }
                }
            },
            Self::EaseInBounce => 1.0 - Self::EaseOutBounce.apply(1.0 - t),
            Self::EaseOutBounce => {
                const N1: f32 = 7.5625;
                const D1: f32 = 2.75;
                if t < 1.0 / D1 {
                    N1 * t * t
                } else if t < 2.0 / D1 {
                    let t = t - 1.5 / D1;
                    N1 * t * t + 0.75
                } else if t < 2.5 / D1 {
                    let t = t - 2.25 / D1;
                    N1 * t * t + 0.9375
                } else {
                    let t = t - 2.625 / D1;
                    N1 * t * t + 0.984375
                }
            },
            Self::EaseInOutBounce => {
                if t < 0.5 {
                    0.5 * (1.0 - Self::EaseOutBounce.apply(1.0 - 2.0 * t))
                } else {
                    0.5 * (1.0 + Self::EaseOutBounce.apply(2.0 * t - 1.0))
                }
            },
        }
    }

    /// Get easing function by name (useful for config files)
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "linear" => Some(Self::Linear),
            "ease-in" | "easein" => Some(Self::EaseIn),
            "ease-out" | "easeout" => Some(Self::EaseOut),
            "ease-in-out" | "easeinout" => Some(Self::EaseInOut),
            "ease-in-quad" | "easeinquad" => Some(Self::EaseInQuad),
            "ease-out-quart" | "easeoutquart" => Some(Self::EaseOutQuart),
            "ease-in-back" | "easeinback" => Some(Self::EaseInBack),
            "ease-out-back" | "easeoutback" => Some(Self::EaseOutBack),
            "ease-in-out-back" | "easeinoutback" => Some(Self::EaseInOutBack),
            "ease-in-elastic" | "easeinelastic" => Some(Self::EaseInElastic),
            "ease-out-elastic" | "easeoutelastic" => Some(Self::EaseOutElastic),
            "ease-in-out-elastic" | "easeinoutelastic" => Some(Self::EaseInOutElastic),
            "ease-in-bounce" | "easeinbounce" => Some(Self::EaseInBounce),
            "ease-out-bounce" | "easeoutbounce" => Some(Self::EaseOutBounce),
            "ease-in-out-bounce" | "easeinoutbounce" => Some(Self::EaseInOutBounce),
            _ => None,
        }
    }

    /// Get the name of the easing function
    pub fn name(&self) -> &'static str {
        match self {
            Self::Linear => "linear",
            Self::EaseIn => "ease-in",
            Self::EaseOut => "ease-out",
            Self::EaseInOut => "ease-in-out",
            Self::EaseInQuad => "ease-in-quad",
            Self::EaseOutQuart => "ease-out-quart",
            Self::EaseInBack => "ease-in-back",
            Self::EaseOutBack => "ease-out-back",
            Self::EaseInOutBack => "ease-in-out-back",
            Self::EaseInElastic => "ease-in-elastic",
            Self::EaseOutElastic => "ease-out-elastic",
            Self::EaseInOutElastic => "ease-in-out-elastic",
            Self::EaseInBounce => "ease-in-bounce",
            Self::EaseOutBounce => "ease-out-bounce",
            Self::EaseInOutBounce => "ease-in-out-bounce",
        }
    }

    /// Get all available easing function names
    pub fn all_names() -> &'static [&'static str] {
        &[
            "linear",
            "ease-in",
            "ease-out",
            "ease-in-out",
            "ease-in-quad",
            "ease-out-quart",
            "ease-in-back",
            "ease-out-back",
            "ease-in-out-back",
            "ease-in-elastic",
            "ease-out-elastic",
            "ease-in-out-elastic",
            "ease-in-bounce",
            "ease-out-bounce",
            "ease-in-out-bounce",
        ]
    }
}

impl Default for EasingFunction {
    fn default() -> Self {
        Self::EaseOut
    }
}

/// Bezier curve easing for custom curves
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BezierEasing {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
}

impl BezierEasing {
    /// Create a new bezier easing curve
    pub fn new(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        Self { x1, y1, x2, y2 }
    }

    /// Sample the cubic bezier curve's X coordinate at parameter t
    ///
    /// Uses the cubic bezier formula: B(t) = (1-t)³P0 + 3(1-t)²tP1 + 3(1-t)t²P2 + t³P3
    /// where P0=(0,0) and P3=(1,1) for CSS cubic-bezier
    #[inline]
    fn sample_curve_x(&self, t: f32) -> f32 {
        // Coefficients derived from bezier formula with P0.x=0, P3.x=1
        let mt = 1.0 - t;
        let mt2 = mt * mt;
        let t2 = t * t;

        3.0 * mt2 * t * self.x1 + 3.0 * mt * t2 * self.x2 + t * t2
    }

    /// Sample the cubic bezier curve's Y coordinate at parameter t
    #[inline]
    fn sample_curve_y(&self, t: f32) -> f32 {
        let mt = 1.0 - t;
        let mt2 = mt * mt;
        let t2 = t * t;

        3.0 * mt2 * t * self.y1 + 3.0 * mt * t2 * self.y2 + t * t2
    }

    /// Calculate derivative of X with respect to parameter t
    ///
    /// Used by Newton-Raphson: derivative tells us the slope at point t
    #[inline]
    fn sample_curve_derivative_x(&self, t: f32) -> f32 {
        let mt = 1.0 - t;
        let mt2 = mt * mt;
        let t2 = t * t;

        3.0 * mt2 * self.x1 + 6.0 * mt * t * (self.x2 - self.x1) + 3.0 * t2 * (1.0 - self.x2)
    }

    /// Solve for parameter t given x coordinate using Newton-Raphson iteration
    ///
    /// Newton-Raphson: t_next = t_current - f(t) / f'(t)
    /// where f(t) = sample_curve_x(t) - target_x
    #[inline]
    fn solve_curve_x(&self, x: f32) -> f32 {
        // Constants based on empirical testing from gre/bezier-easing
        const EPSILON: f32 = 1e-6;
        const MAX_ITERATIONS: usize = 8;

        // Start with x as initial guess (linear approximation)
        let mut t = x;

        for _ in 0..MAX_ITERATIONS {
            let x_error = self.sample_curve_x(t) - x;

            // Converged within tolerance
            if x_error.abs() < EPSILON {
                break;
            }

            let derivative = self.sample_curve_derivative_x(t);

            // Avoid division by zero
            if derivative.abs() < EPSILON {
                break;
            }

            // Newton-Raphson step
            t -= x_error / derivative;

            // Clamp to valid range
            t = t.clamp(0.0, 1.0);
        }

        t
    }

    /// Apply cubic bezier easing to input value
    ///
    /// Given x (time), returns y (eased value).
    /// Uses Newton-Raphson iteration to find parameter t where curve.x(t) = x,
    /// then returns curve.y(t).
    ///
    /// Matches CSS `cubic-bezier(x1, y1, x2, y2)` behavior.
    pub fn apply(&self, x: f32) -> f32 {
        // Handle edge cases for performance and correctness
        if x <= 0.0 {
            return 0.0;
        }
        if x >= 1.0 {
            return 1.0;
        }

        // Special case: linear curve (no need for iteration)
        if (self.x1 - self.y1).abs() < f32::EPSILON && (self.x2 - self.y2).abs() < f32::EPSILON {
            return x;
        }

        // Find t parameter for given x using Newton-Raphson
        let t = self.solve_curve_x(x);

        // Return y value at that t parameter
        self.sample_curve_y(t)
    }
}

/// Common bezier easing presets
impl BezierEasing {
    pub const EASE: BezierEasing = BezierEasing {
        x1: 0.25,
        y1: 0.1,
        x2: 0.25,
        y2: 1.0,
    };

    pub const EASE_IN: BezierEasing = BezierEasing {
        x1: 0.42,
        y1: 0.0,
        x2: 1.0,
        y2: 1.0,
    };

    pub const EASE_OUT: BezierEasing = BezierEasing {
        x1: 0.0,
        y1: 0.0,
        x2: 0.58,
        y2: 1.0,
    };

    pub const EASE_IN_OUT: BezierEasing = BezierEasing {
        x1: 0.42,
        y1: 0.0,
        x2: 0.58,
        y2: 1.0,
    };
}
