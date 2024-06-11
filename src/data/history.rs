use serde::{Deserialize, Serialize};

/// A user's progress through a section.
pub enum SectionProgress {
    /// The user is in the middle of reading this section for the first time.
    Reading {
        /// The last time the user read this section.
        last_read: i64,
        /// The point the user left off on.
        line: usize,
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
        line: usize,
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
    pub fn timestamp(&self) -> i64 {
        match self {
            Self::Reading { last_read, .. }
            | Self::Finished { last_read }
            | Self::Rereading { last_read, .. } => *last_read,
        }
    }

    /// The user's progress through this section by line. Includes the time the section was last read.
    /// 
    /// If the user has never opened this section or has finished it, returns [`None`].
    pub fn line(&self) -> usize {
        match self {
            Self::Reading {
                line,
                ..
            }
            | Self::Rereading {
                line,
                ..
            } => *line,
            Self::Finished { .. } => 0,
        }
    }
}

/// A user's progress through an entry.
pub enum EntryProgress {
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
        line: usize,
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
    pub(super) line: usize,
    pub(super) timestamp: i64,
    pub(super) ever_finished: bool,
}
