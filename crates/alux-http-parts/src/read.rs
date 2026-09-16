//! Reads the parts a body states, from the bytes a caller sent.

use alux_http::{ChunksAlg, PartAlg};
use bytes::Bytes;
use futures::stream;
use multer::{Field, Multipart, parse_boundary};
use std::io::Error as IoError;

/// The parts a body states, read from the bytes a caller sent.
pub struct ReadParts(Multipart<'static>);

impl ReadParts {
    /// Reads the parts a body states, given what the caller said the body was.
    ///
    /// # Errors
    ///
    /// Answers with what went wrong where the media type states no boundary between parts.
    pub fn new(media_type: &str, body: Vec<u8>) -> Result<Self, IoError> {
        let boundary = parse_boundary(media_type).map_err(|error| IoError::other(error.to_string()))?;
        let sent = stream::once(async move { Ok::<Bytes, IoError>(Bytes::from(body)) });

        Ok(Self(Multipart::new(sent, boundary)))
    }
}

impl ChunksAlg for ReadParts {
    type Chunk = ReadPart;
    type Error = IoError;

    async fn next_chunk(&mut self) -> Option<Result<Self::Chunk, Self::Error>> {
        match self.0.next_field().await {
            Ok(field) => {
                let field = field?;
                let media_type = field.content_type().map(ToString::to_string);

                Some(Ok(ReadPart { media_type, field }))
            }
            Err(error) => Some(Err(IoError::other(error.to_string()))),
        }
    }
}

/// One part a body states.
pub struct ReadPart {
    media_type: Option<String>,
    field: Field<'static>,
}

impl PartAlg for ReadPart {
    type Content = ReadPartContent;

    fn part_name(&self) -> Option<&str> {
        self.field.name()
    }

    fn part_file_name(&self) -> Option<&str> {
        self.field.file_name()
    }

    fn part_media_type(&self) -> Option<&str> {
        self.media_type.as_deref()
    }

    fn part_content(self) -> Self::Content {
        ReadPartContent(Some(self.field))
    }
}

/// What one part carries, as the chunks it arrives in.
pub struct ReadPartContent(Option<Field<'static>>);

impl ChunksAlg for ReadPartContent {
    type Chunk = Vec<u8>;
    type Error = IoError;

    async fn next_chunk(&mut self) -> Option<Result<Self::Chunk, Self::Error>> {
        let field = self.0.take()?;
        let read = field.bytes().await.map_err(|error| IoError::other(error.to_string()));

        Some(read.map(|bytes| bytes.to_vec()))
    }
}
