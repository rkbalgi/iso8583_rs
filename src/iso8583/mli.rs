//! This module contains implementation of various MLI types associated with a ISO message
use crate::iso8583::IsoError;
use byteorder::{ByteOrder, WriteBytesExt, ReadBytesExt};
use std::io::Read;


pub enum MLIType {
    MLI2E,
    MLI2I,
    MLI4E,
    MLI4I,
}

pub trait MLI: Sync + Send {
    fn parse(&self, in_buf: &mut Vec<u8>) -> Result<u32, IsoError>;
    fn parse_from_reader(&self, in_buf: &mut dyn Read) -> Result<u32, IsoError>;
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
    fn parse(&self, in_buf: &mut Vec<u8>) -> Result<u32, IsoError>
    {
        let n = byteorder::BigEndian::read_u16(in_buf);
        in_buf.drain(0..2).for_each(|f| drop(f));
        Ok(n as u32)
    }

    fn parse_from_reader(&self, in_buf: &mut dyn Read) -> Result<u32, IsoError> {
        let mut data: Vec<u8> = vec![0; 2];
        match in_buf.read_exact(&mut data[..]) {
            Ok(_) => self.parse(&mut data),
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
    fn parse(&self, in_buf: &mut Vec<u8>) -> Result<u32, IsoError> {
        let n = byteorder::BigEndian::read_u32(in_buf);
        in_buf.drain(0..4).for_each(|f| drop(f));
        Ok(n)
    }


    fn parse_from_reader(&self, in_buf: &mut dyn Read) -> Result<u32, IsoError> {
        let mut data: Vec<u8> = vec![0; 4];
        match in_buf.read_exact(&mut data[..]) {
            Ok(_) => self.parse(&mut data),
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
    fn parse(&self, in_buf: &mut Vec<u8>) -> Result<u32, IsoError>
    {
        let n = byteorder::BigEndian::read_u16(in_buf);
        in_buf.drain(0..2).for_each(|f| drop(f));
        Ok((n - 2) as u32)
    }

    fn parse_from_reader(&self, in_buf: &mut dyn Read) -> Result<u32, IsoError> {
        let mut data: Vec<u8> = vec![0; 2];
        match in_buf.read_exact(&mut data[..]) {
            Ok(_) => self.parse(&mut data),
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
    fn parse(&self, in_buf: &mut Vec<u8>) -> Result<u32, IsoError> {
        let n = byteorder::BigEndian::read_u32(in_buf);
        in_buf.drain(0..4).for_each(|f| drop(f));
        Ok(n - 4)
    }

    fn parse_from_reader(&self, in_buf: &mut dyn Read) -> Result<u32, IsoError> {
        let mut data: Vec<u8> = vec![0; 4];
        match in_buf.read_exact(&mut data[..]) {
            Ok(_) => self.parse(&mut data),
            Err(e) => Err(IsoError { msg: e.to_string() })
        }
    }

    fn create(&self, n: &usize) -> Result<Vec<u8>, IsoError> {
        let mut mli = Vec::<u8>::new();
        let _ = mli.write_u32::<byteorder::BigEndian>((n.clone() as u32) + 4);
        Ok(mli)
    }
}