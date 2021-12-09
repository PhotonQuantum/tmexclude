use std::borrow::Borrow;
use std::ops::{Add, AddAssign};
use std::path::PathBuf;
use std::ptr;

use core_foundation::base::{TCFType, ToVoid};
use core_foundation::number::{kCFBooleanFalse, kCFBooleanTrue};
use core_foundation::url;
use core_foundation::url::kCFURLIsExcludedFromBackupKey;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

#[derive(Debug, Clone, Default)]
pub struct ExclusionActionBatch {
    pub add: Vec<PathBuf>,
    pub remove: Vec<PathBuf>,
}

impl ExclusionActionBatch {
    pub fn apply(self, remove: bool) {
        self.add.into_par_iter().for_each(|path| {
            ExclusionAction::Add(path).apply();
        });
        if remove {
            self.remove.into_par_iter().for_each(|path| {
                ExclusionAction::Remove(path).apply();
            });
        }
    }
}

impl<T> From<T> for ExclusionActionBatch
where
    T: Iterator<Item = ExclusionAction>,
{
    fn from(it: T) -> Self {
        let mut this = Self::default();
        it.for_each(|item| match item {
            ExclusionAction::Add(path) => this.add.push(path),
            ExclusionAction::Remove(path) => this.remove.push(path),
        });
        this
    }
}

impl<T: Borrow<Self>> Add<T> for ExclusionActionBatch {
    type Output = Self;

    fn add(mut self, rhs: T) -> Self::Output {
        self.add.extend_from_slice(&rhs.borrow().add);
        self.remove.extend_from_slice(&rhs.borrow().remove);
        self
    }
}

impl AddAssign for ExclusionActionBatch {
    fn add_assign(&mut self, rhs: Self) {
        self.add.extend_from_slice(&rhs.add);
        self.remove.extend_from_slice(&rhs.remove);
    }
}

#[derive(Debug, Clone)]
pub enum ExclusionAction {
    Add(PathBuf),
    Remove(PathBuf),
}

impl ExclusionAction {
    pub fn apply(self) {
        let value = unsafe {
            if matches!(self, Self::Add(_)) {
                kCFBooleanTrue
            } else {
                kCFBooleanFalse
            }
        };
        match self {
            Self::Add(path) | Self::Remove(path) => {
                if let Some(path) = url::CFURL::from_path(path, false) {
                    unsafe {
                        url::CFURLSetResourcePropertyForKey(
                            path.as_concrete_TypeRef(),
                            kCFURLIsExcludedFromBackupKey,
                            value.to_void(),
                            ptr::null_mut(),
                        );
                    }
                }
            }
        }
    }
}
