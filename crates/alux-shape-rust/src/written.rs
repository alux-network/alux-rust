//! What a generated layout is compiled against.
//!
//! A term states how a value is written; a layout keeps the value and names that writing with a
//! wrapper. These are those wrappers — the runtime counterpart of [`crate::RustShape`]'s `hex` and
//! `decimal`, so a generated declaration compiles without anything else being written by hand.

use serde::de::{Error as DeError, Unexpected};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::Display;
use std::str::FromStr;

/// Bytes written as `0x`-prefixed hexadecimal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Hex<T>(pub T);

/// Writes bytes as the hexadecimal text a shape states.
fn write_hex<S>(bytes: &[u8], writer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut text = String::with_capacity(2 + bytes.len() * 2);
    text.push_str("0x");

    for byte in bytes {
        text.push_str(&format!("{byte:02x}"));
    }

    writer.serialize_str(&text)
}

/// Reads the bytes hexadecimal text states.
fn read_hex<E>(text: &str) -> Result<Vec<u8>, E>
where
    E: DeError,
{
    let digits = text
        .strip_prefix("0x")
        .ok_or_else(|| E::invalid_value(Unexpected::Str(text), &"hexadecimal text, `0x`-prefixed"))?;

    if digits.len() % 2 != 0 {
        return Err(E::invalid_length(digits.len(), &"an even number of hexadecimal digits"));
    }

    (0..digits.len() / 2)
        .map(|index| {
            u8::from_str_radix(&digits[index * 2..index * 2 + 2], 16)
                .map_err(|_| E::invalid_value(Unexpected::Str(text), &"hexadecimal digits"))
        })
        .collect()
}

impl<const N: usize> Serialize for Hex<[u8; N]> {
    fn serialize<S>(&self, writer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        write_hex(&self.0, writer)
    }
}

impl<'de, const N: usize> Deserialize<'de> for Hex<[u8; N]> {
    fn deserialize<D>(reader: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let text = String::deserialize(reader)?;
        let bytes = read_hex::<D::Error>(&text)?;

        bytes.try_into().map(Hex).map_err(|bytes: Vec<u8>| D::Error::invalid_length(bytes.len(), &"the stated length"))
    }
}

impl Serialize for Hex<Vec<u8>> {
    fn serialize<S>(&self, writer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        write_hex(&self.0, writer)
    }
}

impl<'de> Deserialize<'de> for Hex<Vec<u8>> {
    fn deserialize<D>(reader: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let text = String::deserialize(reader)?;

        read_hex::<D::Error>(&text).map(Hex)
    }
}

/// A number written as decimal text, for a width JSON carries no integer of.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Decimal<T>(pub T);

impl<T> Serialize for Decimal<T>
where
    T: Display,
{
    fn serialize<S>(&self, writer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        writer.collect_str(&self.0)
    }
}

impl<'de, T> Deserialize<'de> for Decimal<T>
where
    T: FromStr,
    T::Err: Display,
{
    fn deserialize<D>(reader: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let text = String::deserialize(reader)?;

        text.parse().map(Decimal).map_err(D::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bytes_are_written_and_read_as_hexadecimal() {
        let value = Hex([0xbe_u8, 0xef]);
        let json = serde_json::to_string(&value).expect("writes");

        assert_eq!(json, "\"0xbeef\"");
        assert_eq!(serde_json::from_str::<Hex<[u8; 2]>>(&json).expect("reads"), value);
    }

    #[test]
    fn a_wide_number_is_written_and_read_as_decimal_text() {
        let value = Decimal(u128::from(u64::MAX) + 1);
        let json = serde_json::to_string(&value).expect("writes");

        assert_eq!(json, "\"18446744073709551616\"");
        assert_eq!(serde_json::from_str::<Decimal<u128>>(&json).expect("reads"), value);
    }

    #[test]
    fn text_that_is_not_hexadecimal_is_not_read() {
        assert!(serde_json::from_str::<Hex<[u8; 2]>>("\"beef\"").is_err());
        assert!(serde_json::from_str::<Hex<[u8; 2]>>("\"0xbe\"").is_err());
    }
}
