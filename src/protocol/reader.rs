//! Checked payload consumption shared by parameter reply decoders.
use super::{DataType, Value};
use crate::Error;

pub(super) struct PayloadReader<'a> {
    remaining: &'a [u8],
}
impl<'a> PayloadReader<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { remaining: bytes }
    }
    pub fn take(&mut self, width: usize, error: &'static str) -> Result<&'a [u8], Error> {
        let bytes = self
            .remaining
            .get(..width)
            .ok_or(Error::InvalidResponse(error))?;
        self.remaining = &self.remaining[width..];
        Ok(bytes)
    }
    pub fn expect(&mut self, expected: &[u8], error: &'static str) -> Result<(), Error> {
        if self.take(expected.len(), error)? != expected {
            return Err(Error::InvalidResponse(error));
        }
        Ok(())
    }
    pub fn value(&mut self, kind: DataType, error: &'static str) -> Result<Value, Error> {
        Value::decode(kind, self.take(kind.width(), error)?)
    }
    pub fn finish(self, error: &'static str) -> Result<(), Error> {
        if !self.remaining.is_empty() {
            return Err(Error::InvalidResponse(error));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oversized_read_fails_without_advancing_or_overflowing() {
        let mut reader = PayloadReader::new(&[1, 2]);
        assert!(reader.take(usize::MAX, "truncated").is_err());
        assert_eq!(reader.take(2, "truncated").unwrap(), [1, 2]);
        reader.finish("trailing").unwrap();
    }
}
