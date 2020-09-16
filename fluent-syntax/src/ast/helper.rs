#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::Comment;
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
pub enum CommentDef<S> {
    Single { content: S },
    Multi { content: Vec<S> },
}

impl<'s, S> From<CommentDef<S>> for Comment<S> {
    fn from(input: CommentDef<S>) -> Self {
        match input {
            CommentDef::Single { content } => Comment {
                content: vec![content],
            },
            CommentDef::Multi { content } => Comment { content },
        }
    }
}
