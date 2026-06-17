#[cfg(test)]
use clap::CommandFactory;

use crate::*;

impl From<SyncArg> for gwz_core::SyncBehavior {
    fn from(value: SyncArg) -> Self {
        match value {
            SyncArg::FetchOnly => gwz_core::SyncBehavior::FetchOnly,
            SyncArg::FfOnly => gwz_core::SyncBehavior::FfOnly,
            SyncArg::Merge => gwz_core::SyncBehavior::Merge,
            SyncArg::Rebase => gwz_core::SyncBehavior::Rebase,
            SyncArg::Reset => gwz_core::SyncBehavior::Reset,
            SyncArg::DriverSelected => gwz_core::SyncBehavior::DriverSelected,
        }
    }
}
