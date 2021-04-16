use std::{
    convert::TryInto,
    ops::{Deref, DerefMut},
    str,
};

pub(crate) fn validate_char(c: char) -> bool {
    matches!(c, '0'..='9' | 'a'..='z' | 'A'..='Z' | '_' | '-')
}

/// A Id describing a Video or Playlist.
///
/// Like [`String`], the contents  have to be valid UTF-8 at all times.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Id<const N: usize> {
    data: [u8; N],
}

impl<const N: usize> std::fmt::Display for Id<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&*self)
    }
}

#[derive(thiserror::Error, Debug, Clone, Copy)]
#[error("Expected a string of length {expected} but found a string of length {found}")]
pub struct Error {
    pub(crate) expected: usize,
    pub(crate) found: usize,
}

impl<const N: usize> std::str::FromStr for Id<N> {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            data: value.as_bytes().try_into().map_err(|_| Error {
                expected: N,
                found: value.len(),
            })?,
        })
    }
}

impl<const N: usize> Deref for Id<N> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        str::from_utf8(&self.data[..]).expect("Id was invalid UTF-8")
    }
}

impl<const N: usize> DerefMut for Id<N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        str::from_utf8_mut(&mut self.data[..]).expect("Id was invalid UTF-8")
    }
}
