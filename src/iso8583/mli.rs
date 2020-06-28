//! This module contains implementation of various MLI types associated with a ISO message
use crate::iso8583::IsoError;
use byteorder::{WriteBytesExt, ReadBytesExt};
use std::io::Read;


pub enum MLIType {
    MLI2E,
    MLI2I,
    MLI4E,
    MLI4I,
}

pub trait MLI: Sync + Send {
    /// Extracts MLI from in_buf
    fn parse(&self, in_buf: &mut dyn Read) -> Result<u32, IsoError>;
    /// Creates a Vec<u8> that represents the MLI containing n bytes
    fn create(&self, n: &usize) -> Result<Vec<u8>, IsoError>;
}

/// This struct represents an MLI of 2E (i.e 2 bytes of length indicator exclusive of its own length)
pub struct MLI2E {}

/// This struct represents an MLI of 2I (i.e 2 bytes of length indicator inclusive of its own length)
pub struct MLI2I {}

/// This struct represents an MLI of 4E (i.e 4 bytes of length indicator exclusive of its own length)
pub struct MLI4E {}

/// This struct represents an MLI of 4I (i.e 4 bytes of length indicator inclusive of its own length)
pub struct MLI4I {}


impl MLI for MLI2E {
    fn parse(&self, in_buf: &mut dyn Read) -> Result<u32, IsoError> {
        match in_buf.read_u16::<byteorder::BigEndian>() {
            Ok(n) => {
                Ok(n as u32)
            }
            Err(e) => Err(IsoError { msg: e.to_string() })
        }
    }

    fn create(&self, n: &usize) -> Result<Vec<u8>, IsoError> {
        let mut mli = Vec::<u8>::new();
        let _ = mli.write_u16::<byteorder::BigEndian>(n.clone() as u16);
        Ok(mli)
    }
}


impl MLI for MLI4E {
    fn parse(&self, in_buf: &mut dyn Read) -> Result<u32, IsoError> {
        match in_buf.read_u32::<byteorder::BigEndian>() {
            Ok(n) => Ok(n),
            Err(e) => Err(IsoError { msg: e.to_string() })
        }
    }

    fn create(&self, n: &usize) -> Result<Vec<u8>, IsoError> {
        let mut mli = Vec::<u8>::new();
        let _ = mli.write_u32::<byteorder::BigEndian>(n.clone() as u32);
        Ok(mli)
    }
}


impl MLI for MLI2I {
    fn parse(&self, in_buf: &mut dyn Read) -> Result<u32, IsoError> {
        match in_buf.read_u16::<byteorder::BigEndian>() {
            Ok(n) => Ok((n - 2) as u32),
            Err(e) => Err(IsoError { msg: e.to_string() })
        }
    }

    fn create(&self, n: &usize) -> Result<Vec<u8>, IsoError> {
        let mut mli = Vec::<u8>::new();
        let _ = mli.write_u16::<byteorder::BigEndian>((n.clone() as u16) + 2);
        Ok(mli)
    }
}

impl MLI for MLI4I {
    fn parse(&self, in_buf: &mut dyn Read) -> Result<u32, IsoError> {
        match in_buf.read_u32::<byteorder::BigEndian>() {
            Ok(n) => Ok(n - 4),
            Err(e) => Err(IsoError { msg: e.to_string() })
        }
    }

    fn create(&self, n: &usize) -> Result<Vec<u8>, IsoError> {
        let mut mli = Vec::<u8>::new();
        let _ = mli.write_u32::<byteorder::BigEndian>((n.clone() as u32) + 4);
        Ok(mli)
    }
}


mod tests {
    #[test]
    fn test_2e() {}
    #[test]
    fn test_2i() {}
    #[test]
    fn test_4e() {}
    #[test]
    fn test_4i() {}

}