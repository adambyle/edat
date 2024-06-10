use serde::{Deserialize, Serialize};

/// A user's progress through a section.
pub enum SectionProgress {
    /// The user has never opened this section.
    Unstarted,

    /// The user is in the middle of reading this section for the first time.
    Reading {
        /// The last time the user read this section.
        last_read: i64,
        /// The point the user left off on.
        progress: usize,
    },

    /// The user has finished reading this section.
    Finished {
        /// The time the user finished reading this section.
        last_read: i64,
    },

    /// The user is in the middle of rereading this section.
    Rereading {
        /// The last time the user read this section.
        last_read: i64,
        /// The point the user left off on.
        progress: usize,
    },
}

impl SectionProgress {
    /// Whether the user has ever made it to the end of this section.
    pub fn ever_finished(&self) -> bool {
        matches!(self, Self::Finished { .. } | Self::Rereading { .. })
    }

    /// The time the user last opened this section.
    ///
    /// Returns [`None`] if the user has never opened this section.
    pub fn timestamp(&self) -> Option<i64> {
        match self {
            Self::Unstarted => None,
            Self::Reading { last_read, .. }
            | Self::Finished { last_read }
            | Self::Rereading { last_read, .. } => Some(*last_read),
        }
    }

    /// The user's progress through this section by line. Includes the time the section was last read.
    /// 
    /// If the user has never opened this section or has finished it, returns [`None`].
    pub fn progress(&self) -> Option<(usize, i64)> {
        match self {
            Self::Reading {
                progress,
                last_read,
            }
            | Self::Rereading {
                progress,
                last_read,
            } => Some((*progress, *last_read)),
            _ => None,
        }
    }

    /// Whether the user has started reading this section.
    pub fn started(&self) -> bool {
        !matches!(self, Self::Unstarted)
    }
}

/// A user's progress through an entry.
pub enum EntryProgress {
    /// None of this entry's sections have been opened.
    Unstarted,

    /// Sections have been completely read up to a certain unstarted section.
    UpToSection {
        /// The id of the unread section.
        section_id: u32,
        /// The index of the unread section in the entry.
        section_index: usize,
        /// The total number of sections in the entry.
        out_of: usize,
    },

    /// The user is in the middle of reading this section.
    InSection {
        /// The id of the section that the user is in.
        section_id: u32,
        /// The index of the section in the entry.
        section_index: usize,
        /// The total number of sections in the entry.
        out_of: usize,
        /// The point the user left off on.
        progress: usize,
        /// The time the user last opened this section.
        last_read: i64,
    },

    /// The user finished reading every section in this entry.
    Finished {
        /// The time the user finished reading this entry.
        last_read: i64,
    },
}

#[derive(Clone, Serialize, Deserialize)]
pub(super) struct HistoryEntry {
    pub(super) section: u32,
    pub(super) progress: usize,
    pub(super) timestamp: i64,
    pub(super) ever_finished: bool,
}
