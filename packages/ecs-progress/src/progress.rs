use bevy::prelude::*;
use derive_more::derive::{Add, AddAssign, Deref, DerefMut, Sub, SubAssign};

/// Represents the progress that is being tracked.
///
/// It indicates how much work has been completed and how much is left to do.
/// When the value of `done` reaches the value of `total`, it is considered
/// complete.
///
/// For your convenience, you can easily convert `bool`s into this type.
/// You can also convert `Progress` values into floats in the `0.0..=1.0` range.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Event)]
#[derive(Add, AddAssign, Sub, SubAssign)]
pub struct Progress {
    /// The units of work that have been completed.
    pub done: u32,
    /// The total units of work expected.
    pub total: u32,
}

impl Progress {
    /// Returns true if `done` has reached `total`
    pub fn is_complete(self) -> bool {
        self.done >= self.total
    }

    /// Returns the progress as a fraction (0.0 to 1.0)
    pub fn fraction(self) -> f32 {
        if self.total == 0 {
            1.0
        } else {
            (self.done as f32 / self.total as f32).min(1.0)
        }
    }
}

impl From<bool> for Progress {
    fn from(b: bool) -> Progress {
        Progress {
            total: 1,
            done: b as u32,
        }
    }
}

impl From<Progress> for f32 {
    fn from(p: Progress) -> f32 {
        p.fraction()
    }
}

impl From<Progress> for f64 {
    fn from(p: Progress) -> f64 {
        p.fraction() as f64
    }
}

/// Represents progress that is intended to be "hidden" from the user.
///
/// Such progress must be completed in order to advance state (or generally
/// consider everything to be ready), but is not intended to be shown in UI
/// progress bars or other user-facing progress indicators.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Event)]
#[derive(Add, AddAssign, Sub, SubAssign)]
#[derive(Deref, DerefMut)]
pub struct HiddenProgress(pub Progress);

impl HiddenProgress {
    /// Returns true if the underlying progress is complete
    pub fn is_complete(self) -> bool {
        self.0.is_complete()
    }

    /// Returns the progress as a fraction (0.0 to 1.0)
    pub fn fraction(self) -> f32 {
        self.0.fraction()
    }
}

impl From<Progress> for HiddenProgress {
    fn from(value: Progress) -> Self {
        Self(value)
    }
}

impl From<HiddenProgress> for Progress {
    fn from(value: HiddenProgress) -> Self {
        value.0
    }
}

impl From<bool> for HiddenProgress {
    fn from(b: bool) -> HiddenProgress {
        Progress::from(b).into()
    }
}

impl From<HiddenProgress> for f32 {
    fn from(p: HiddenProgress) -> f32 {
        f32::from(p.0)
    }
}

impl From<HiddenProgress> for f64 {
    fn from(p: HiddenProgress) -> f64 {
        f64::from(p.0)
    }
}

/// Event fired when all progress (visible and hidden) is complete for a given
/// state
#[derive(Event, Debug, Clone)]
pub struct ProgressComplete<S: States> {
    /// The state that has completed its progress
    pub state: S,
}

/// Event containing progress updates for debugging/monitoring
#[derive(Event, Debug, Clone)]
pub struct ProgressUpdate<S: States> {
    /// Visible progress
    pub visible: Progress,
    /// Hidden progress  
    pub hidden: HiddenProgress,
    /// Whether this update indicates completion
    pub is_complete: bool,
    /// State associated with this progress
    pub state: S,
}
