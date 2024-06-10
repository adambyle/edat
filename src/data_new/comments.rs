use serde::{Deserialize, Serialize};

use super::*;

#[derive(Serialize, Deserialize)]
pub(super) struct CommentData {
    pub(super) uuid: u128,
    pub(super) content: Vec<String>,
    pub(super) show: bool,
    pub(super) line: usize,
    pub(super) author: String,
    pub(super) timestamp: i64,
}

/// A comment thread.
pub struct Thread<'index> {
    /// The line the comments are attached to.
    pub line: usize,

    /// The comments added to this line.
    pub comments: Vec<Comment<'index>>,
}

/// A comment in a thread.
pub struct Comment<'index> {
    /// A unique identifier for this comment.
    pub uuid: u128,

    /// A list of bodies that this comment has had. When the comment is edited,
    /// the new body is pushed onto the list. The last item is the most recent
    /// and the one that is shown.
    pub content: &'index [String],

    /// Whether to show the content of this comment. If `false`, the author has
    /// chosen to remove it.
    pub show: bool,

    /// The user who authored this comment.
    pub author: User<'index>,

    /// When the comment was originally written. Edits do not change this.
    pub timestamp: i64,
}
