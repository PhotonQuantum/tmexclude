use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum ExclusionAction {
    Add(PathBuf),
    Remove(PathBuf),
}

impl ExclusionAction {
    pub fn flip(self) -> Self {
        match self {
            ExclusionAction::Add(p) => ExclusionAction::Remove(p),
            ExclusionAction::Remove(p) => ExclusionAction::Add(p),
        }
    }
}
