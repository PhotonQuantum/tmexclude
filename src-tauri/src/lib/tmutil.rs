//! Utils needed to operate on `TimeMachine`.
use std::borrow::Borrow;
use std::collections::HashMap;
use std::ops::{Add, AddAssign};
use std::path::{Path, PathBuf};
use std::ptr;

use core_foundation::base::{CFTypeRef, TCFType, ToVoid};
use core_foundation::error::{CFError, CFErrorRef};
use core_foundation::number::{kCFBooleanFalse, kCFBooleanTrue};
use core_foundation::string::CFStringRef;
use core_foundation::url;
use core_foundation::url::{kCFURLIsExcludedFromBackupKey, CFURL};
use serde::{Deserialize, Serialize};
use tap::TapFallible;
use tracing::{info, warn};
use ts_rs::TS;

use crate::error::ApplyError;

/// Check whether a path is excluded from time machine.
///
/// # Errors
/// `io::Error` if can't query xattr of given file.
pub fn is_excluded(path: impl AsRef<Path>) -> std::io::Result<bool> {
    let path = path.as_ref();
    Ok(
        xattr::get(path, "com.apple.metadata:com_apple_backup_excludeItem")
            .tap_err(|e| warn!("Error when querying xattr of file {:?}: {}", path, e))?
            .is_some(),
    )
}

/// Represents a batch of tmutil modifications.
#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../src/bindings/")]
pub struct ExclusionActionBatch {
    /// Paths to be added to backup exclusion list.
    pub add: Vec<PathBuf>,
    /// Paths to be removed from backup exclusion list.
    pub remove: Vec<PathBuf>,
}

impl ExclusionActionBatch {
    /// Return `true` if the batch contains no actions.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.add.is_empty() && self.remove.is_empty()
    }
    /// Return the actions count in the batch.
    #[must_use]
    pub fn count(&self) -> usize {
        self.add.len() + self.remove.len()
    }
    /// Apply the batch.
    pub fn apply(self) -> Result<(), HashMap<PathBuf, ApplyError>> {
        let errors: HashMap<_, _> = self
            .add
            .into_iter()
            .filter_map(|path| {
                info!("Excluding {:?} from backups", path);
                ExclusionAction::Add(path.clone())
                    .apply()
                    .err()
                    .map(|e| (path, e))
            })
            .chain(self.remove.into_iter().filter_map(|path| {
                info!("Including {:?} in backups", path);
                ExclusionAction::Remove(path.clone())
                    .apply()
                    .err()
                    .map(|e| (path, e))
            }))
            .collect();
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
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

/// Represents a tmutil modification.
#[derive(Debug, Clone)]
pub enum ExclusionAction {
    /// Add a path to backup exclusion list.
    Add(PathBuf),
    /// Remove a path to backup exclusion list.
    Remove(PathBuf),
}

impl ExclusionAction {
    /// Apply the action.
    pub fn apply(self) -> Result<(), ApplyError> {
        let value = unsafe {
            if matches!(self, Self::Add(_)) {
                kCFBooleanTrue
            } else {
                kCFBooleanFalse
            }
        };
        match self {
            Self::Add(path) | Self::Remove(path) => {
                if let Some(path) = CFURL::from_path(path, false) {
                    Ok(set_resource_property_for_key(
                        &path,
                        unsafe { kCFURLIsExcludedFromBackupKey },
                        value.to_void(),
                    )?)
                } else {
                    Err(ApplyError::InvalidURL)
                }
            }
        }
    }
}

fn set_resource_property_for_key(
    url: &CFURL,
    key: CFStringRef,
    value: CFTypeRef,
) -> Result<(), CFError> {
    let mut err: CFErrorRef = ptr::null_mut();
    let result = unsafe {
        url::CFURLSetResourcePropertyForKey(url.as_concrete_TypeRef(), key, value, &mut err)
    };
    if result == 0 {
        let err = unsafe { CFError::wrap_under_create_rule(err) };
        Err(err)
    } else {
        Ok(())
    }
}
