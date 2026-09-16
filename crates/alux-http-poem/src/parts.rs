//! Reads a body arriving as parts with what Poem reads one with.

use alux_http::{ChunksAlg, PartAlg};
use poem::web::{Field, Multipart};
use std::io::Error as IoError;

/// The parts Poem read, as the sequence they arrive in.
pub struct PoemParts(pub(crate) Multipart);

impl ChunksAlg for PoemParts {
    type Chunk = PoemPart;
    type Error = IoError;

    async fn next_chunk(&mut self) -> Option<Result<Self::Chunk, Self::Error>> {
        match self.0.next_field().await {
            Ok(field) => Some(Ok(PoemPart(field?))),
            Err(error) => Some(Err(IoError::other(error.to_string()))),
        }
    }
}

/// One part Poem read.
pub struct PoemPart(Field);

impl PartAlg for PoemPart {
    type Content = PoemPartContent;

    fn part_name(&self) -> Option<&str> {
        self.0.name()
    }

    fn part_file_name(&self) -> Option<&str> {
        self.0.file_name()
    }

    fn part_media_type(&self) -> Option<&str> {
        self.0.content_type()
    }

    fn part_content(self) -> Self::Content {
        PoemPartContent(Some(self.0))
    }
}

/// What one part carries, as the chunks it arrives in.
///
/// Poem reads a field whole, so this is a sequence of one. What a part carries is still stated as a
/// sequence, because that is what it is wherever a framework reads it a piece at a time.
pub struct PoemPartContent(Option<Field>);

impl ChunksAlg for PoemPartContent {
    type Chunk = Vec<u8>;
    type Error = IoError;

    async fn next_chunk(&mut self) -> Option<Result<Self::Chunk, Self::Error>> {
        let field = self.0.take()?;

        Some(field.bytes().await.map_err(|error| IoError::other(error.to_string())))
    }
}
