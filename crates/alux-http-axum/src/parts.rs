//! Reads a body arriving as parts with what axum reads one with.

use alux_http::{ChunksAlg, PartAlg};
use axum::extract::Multipart;
use axum::extract::multipart::Field;
use std::io::Error as IoError;

/// The parts axum read, as the sequence they arrive in.
pub struct AxumParts(pub(crate) Multipart);

impl ChunksAlg for AxumParts {
    type Chunk = AxumPart;
    type Error = IoError;

    async fn next_chunk(&mut self) -> Option<Result<Self::Chunk, Self::Error>> {
        // axum lends a part out of the reader it came from, so a part is taken whole as it arrives.
        match self.0.next_field().await {
            Ok(field) => Some(AxumPart::taken(field?).await),
            Err(error) => Some(Err(IoError::other(error.to_string()))),
        }
    }
}

/// One part axum read.
pub struct AxumPart {
    name: Option<String>,
    file_name: Option<String>,
    media_type: Option<String>,
    content: Vec<u8>,
}

impl AxumPart {
    async fn taken(field: Field<'_>) -> Result<Self, IoError> {
        let name = field.name().map(ToOwned::to_owned);
        let file_name = field.file_name().map(ToOwned::to_owned);
        let media_type = field.content_type().map(ToOwned::to_owned);
        let content = field.bytes().await.map_err(|error| IoError::other(error.to_string()))?;

        Ok(Self { name, file_name, media_type, content: content.to_vec() })
    }
}

impl PartAlg for AxumPart {
    type Content = AxumPartContent;

    fn part_name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    fn part_file_name(&self) -> Option<&str> {
        self.file_name.as_deref()
    }

    fn part_media_type(&self) -> Option<&str> {
        self.media_type.as_deref()
    }

    fn part_content(self) -> Self::Content {
        AxumPartContent(Some(self.content))
    }
}

/// What one part carries, as the chunks it arrives in.
pub struct AxumPartContent(Option<Vec<u8>>);

impl ChunksAlg for AxumPartContent {
    type Chunk = Vec<u8>;
    type Error = IoError;

    async fn next_chunk(&mut self) -> Option<Result<Self::Chunk, Self::Error>> {
        self.0.take().map(Ok)
    }
}
