//! States a body that arrives as parts, rather than one that arrives whole.

use core::future::Future;

/// States one part of a body that arrives as parts.
///
/// A part states what it was sent under, and it carries a body like any other. What it carries is
/// produced over time, so reading it is [`ChunksAlg`](crate::ChunksAlg) again, one level down: a
/// body of parts is a sequence, and so is each part's content.
pub trait PartAlg {
    /// What this part carries, read as the chunks it arrives in.
    type Content;

    /// Returns the name this part was sent under, where it states one.
    fn part_name(&self) -> Option<&str>;

    /// Returns the file name this part was sent under, where it states one.
    fn part_file_name(&self) -> Option<&str>;

    /// Returns the media type this part states, where it states one.
    fn part_media_type(&self) -> Option<&str>;

    /// Reads what this part carries.
    fn part_content(self) -> Self::Content;
}

/// States how an argument is read from a body that arrives as parts.
///
/// This is the reading counterpart of a body answered over time. There, a domain states what it
/// produces and an interpretation carries it; here, an interpretation produces the parts and a
/// domain states what it makes of them. `Parts` is whichever reader the interpretation has, so a
/// type stating this once is read the same way by every one of them.
pub trait FromPartsAlg<Parts>: Sized {
    /// What reading this argument states when the parts do not state it.
    type Error;

    /// Reads this argument from the parts a caller sent.
    fn from_parts(parts: Parts) -> impl Future<Output = Result<Self, Self::Error>> + Send;
}
