//! This module contains implementation of various MLI types associated with a ISO message
use crate::iso8583::IsoError;
use byteorder::{ByteOrder, WriteBytesExt};

pub trait MLI: Sync + Send {
    /// Parses and returns a u32 that is equal to the number of bytes
    /// in the message or an IsoError if there are insufficient bytes etc
    fn parse(&self, in_buf: &mut Vec<u8>) -> Result<u32, IsoError>;

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
    fn parse(&self, in_buf: &mut Vec<u8>) -> Result<u32, IsoError> {
        if in_buf.len() >= 2 {
            trace!("while reading mli .. {}", hex::encode(&in_buf.as_slice()));
            let mli = byteorder::BigEndian::read_u16(&in_buf[0..2]);
            in_buf.drain(0..2 as usize).for_each(drop);
            return Ok(mli as u32);
        }
        return Err(IsoError { msg: "insufficient bytes in buf".to_string() });
    }

    fn create(&self, n: &usize) -> Result<Vec<u8>, IsoError> {
        let mut mli = Vec::<u8>::new();
        let _ = mli.write_u16::<byteorder::BigEndian>(n.clone() as u16);
        Ok(mli)
    }
}

impl MLI for MLI4E {
    fn parse(&self, in_buf: &mut Vec<u8>) -> Result<u32, IsoError> {
        if in_buf.len() >= 4 {
            trace!("while reading mli .. {}", hex::encode(&in_buf.as_slice()));
            let mli = byteorder::BigEndian::read_u32(&in_buf[0..4]);
            in_buf.drain(0..4 as usize).for_each(drop);
            return Ok(mli as u32);
        }
        return Err(IsoError { msg: "insufficient bytes in buf".to_string() });
    }

    fn create(&self, n: &usize) -> Result<Vec<u8>, IsoError> {
        let mut mli = Vec::<u8>::new();
        let _ = mli.write_u32::<byteorder::BigEndian>(n.clone() as u32);
        Ok(mli)
    }
}


impl MLI for MLI2I {
    fn parse(&self, in_buf: &mut Vec<u8>) -> Result<u32, IsoError> {
        if in_buf.len() >= 2 {
            trace!("while reading mli .. {}", hex::encode(&in_buf.as_slice()));
            let mli = byteorder::BigEndian::read_u16(&in_buf[0..2]);
            in_buf.drain(0..2 as usize).for_each(drop);
            return Ok((mli as u32) - 2);
        }
        return Err(IsoError { msg: "insufficient bytes in buf".to_string() });
    }

    fn create(&self, n: &usize) -> Result<Vec<u8>, IsoError> {
        let mut mli = Vec::<u8>::new();
        let _ = mli.write_u16::<byteorder::BigEndian>((n.clone() as u16) + 2);
        Ok(mli)
    }
}


impl MLI for MLI4I {
    fn parse(&self, in_buf: &mut Vec<u8>) -> Result<u32, IsoError> {
        if in_buf.len() >= 4 {
            trace!("while reading mli .. {}", hex::encode(&in_buf.as_slice()));
            let mli = byteorder::BigEndian::read_u32(&in_buf[0..4]);
            in_buf.drain(0..4 as usize).for_each(drop);
            return Ok((mli as u32) - 4);
        }
        return Err(IsoError { msg: "insufficient bytes in buf".to_string() });
    }

    fn create(&self, n: &usize) -> Result<Vec<u8>, IsoError> {
        let mut mli = Vec::<u8>::new();
        let _ = mli.write_u32::<byteorder::BigEndian>((n.clone() as u32) + 4);
        Ok(mli)
    }
}