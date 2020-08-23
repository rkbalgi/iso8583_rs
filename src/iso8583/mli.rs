//! This module contains implementation of various MLI types associated with a ISO message
use crate::iso8583::IsoError;
use byteorder::{WriteBytesExt, ReadBytesExt};
use std::io::{Read, ErrorKind, Error};
use std::net::{TcpStream};


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
    /// Checks to see if data is available for `MLI::parse`
    fn is_available(&self, stream: &TcpStream) -> Result<bool, IsoError>;
}

/// This struct represents an MLI of 2E (i.e 2 bytes of length indicator exclusive of its own length)
pub struct MLI2E {}

/// This struct represents an MLI of 2I (i.e 2 bytes of length indicator inclusive of its own length)
pub struct MLI2I {}

/// This struct represents an MLI of 4E (i.e 4 bytes of length indicator exclusive of its own length)
pub struct MLI4E {}

/// This struct represents an MLI of 4I (i.e 4 bytes of length indicator inclusive of its own length)
pub struct MLI4I {}

/// convert a std::io::Error into an IsoError
fn convert_err(e: &Error) -> IsoError {
    match e.kind() {
        ErrorKind::ConnectionReset | ErrorKind::UnexpectedEof => {
            IsoError { msg: format!("connection closed. cause: {:?}", e.kind()) }
        }
        _ => {
            IsoError { msg: format!("{:?}: {}", e.kind(), e.to_string()) }
        }
    }
}

impl MLI for MLI2E {
    fn parse(&self, in_buf: &mut dyn Read) -> Result<u32, IsoError> {
        match in_buf.read_u16::<byteorder::BigEndian>() {
            Ok(n) => {
                Ok(n as u32)
            }
            Err(e) => {
                Err(convert_err(&e))
            }
        }
    }

    fn create(&self, n: &usize) -> Result<Vec<u8>, IsoError> {
        let mut mli = Vec::<u8>::new();
        let _ = mli.write_u16::<byteorder::BigEndian>(n.clone() as u16);
        Ok(mli)
    }

    fn is_available(&self, stream: &TcpStream) -> Result<bool, IsoError> {
        let mut buf = vec![0; 2];

        //debug!("{}", stream.bytes().count());

        match stream.peek(&mut buf) {
            Ok(n) => {
                if n == 2 {
                    Ok(true)
                } else {
                    Err(IsoError { msg: format!("client disconnected") })
                }
            }
            Err(e) => Err(IsoError { msg: format!("stream err. cause: {}", e.to_string()) })
        }
    }
}


impl MLI for MLI4E {
    fn parse(&self, in_buf: &mut dyn Read) -> Result<u32, IsoError> {
        match in_buf.read_u32::<byteorder::BigEndian>() {
            Ok(n) => Ok(n),
            Err(e) => {
                Err(convert_err(&e))
            }
        }
    }

    fn create(&self, n: &usize) -> Result<Vec<u8>, IsoError> {
        let mut mli = Vec::<u8>::new();
        let _ = mli.write_u32::<byteorder::BigEndian>(n.clone() as u32);
        Ok(mli)
    }

    fn is_available(&self, stream: &TcpStream) -> Result<bool, IsoError> {
        let mut buf = vec![0; 4];

        match stream.peek(&mut buf) {
            Ok(n) => {
                if n == 4 {
                    Ok(true)
                } else {
                    Err(IsoError { msg: format!("client disconnected") })
                }
            }
            Err(e) => Err(IsoError { msg: format!("stream err. cause: {}", e.to_string()) })
        }
    }
}


impl MLI for MLI2I {
    fn parse(&self, in_buf: &mut dyn Read) -> Result<u32, IsoError> {
        match in_buf.read_u16::<byteorder::BigEndian>() {
            Ok(n) => Ok((n - 2) as u32),
            Err(e) => {
                Err(convert_err(&e))
            }
        }
    }

    fn create(&self, n: &usize) -> Result<Vec<u8>, IsoError> {
        let mut mli = Vec::<u8>::new();
        let _ = mli.write_u16::<byteorder::BigEndian>((n.clone() as u16) + 2);
        Ok(mli)
    }

    fn is_available(&self, stream: &TcpStream) -> Result<bool, IsoError> {
        let mut buf = vec![0; 2];

        match stream.peek(&mut buf) {
            Ok(n) => {
                if n == 2 {
                    Ok(true)
                } else {
                    Err(IsoError { msg: format!("client disconnected") })
                }
            }
            Err(e) => Err(IsoError { msg: format!("stream err. cause: {}", e.to_string()) })
        }
    }
}

impl MLI for MLI4I {
    fn parse(&self, in_buf: &mut dyn Read) -> Result<u32, IsoError> {
        match in_buf.read_u32::<byteorder::BigEndian>() {
            Ok(n) => Ok(n - 4),
            Err(e) => {
                Err(convert_err(&e))
            }
        }
    }

    fn create(&self, n: &usize) -> Result<Vec<u8>, IsoError> {
        let mut mli = Vec::<u8>::new();
        let _ = mli.write_u32::<byteorder::BigEndian>((n.clone() as u32) + 4);
        Ok(mli)
    }

    fn is_available(&self, stream: &TcpStream) -> Result<bool, IsoError> {
        let mut buf = vec![0; 4];

        match stream.peek(&mut buf) {
            Ok(n) => {
                if n == 4 {
                    Ok(true)
                } else {
                    Err(IsoError { msg: format!("client disconnected") })
                }
            }
            Err(e) => Err(IsoError { msg: format!("stream err. cause: {}", e.to_string()) })
        }
    }
}


#[cfg(test)]
mod tests {
    use byteorder::WriteBytesExt;
    use crate::iso8583::mli::{MLI2E, MLI4E, MLI2I, MLI4I};
    use crate::iso8583::mli::MLI;
    use std::io::{Cursor};

    #[test]
    fn test_2e() {
        let msg = "hello world";
        let mut data: Vec<u8> = vec![];
        data.write_u16::<byteorder::BigEndian>(msg.len() as u16);
        data.extend_from_slice(msg.as_bytes());


        let mli: &dyn MLI = &MLI2E {};
        assert_eq!(mli.parse(&mut Cursor::new(data)).unwrap(), 11 as u32);
        assert_eq!(mli.create(&(msg.len() as usize)).unwrap(), vec![0 as u8, 0x0b as u8]);
    }

    #[test]
    fn test_2i() {
        let msg = "hello world";
        let mut data: Vec<u8> = vec![];
        data.write_u16::<byteorder::BigEndian>((msg.len() + 2) as u16);
        data.extend_from_slice(msg.as_bytes());


        let mli: &dyn MLI = &MLI2I {};
        assert_eq!(mli.parse(&mut Cursor::new(data)).unwrap(), 11 as u32);
        assert_eq!(mli.create(&(msg.len() as usize)).unwrap(), vec![0 as u8, 0x0d as u8]);
    }

    #[test]
    fn test_4e() {
        let mut msg = String::new();
        for _ in 0..257 {
            msg.push('a');
        }
        let mut data: Vec<u8> = vec![];
        data.write_u32::<byteorder::BigEndian>(msg.len() as u32);
        data.extend_from_slice(msg.as_bytes());


        let mli: &dyn MLI = &MLI4E {};
        assert_eq!(mli.parse(&mut Cursor::new(data)).unwrap(), 257 as u32);
        assert_eq!(mli.create(&(msg.len() as usize)).unwrap(), vec![0x00, 0x00, 0x01 as u8, 0x01 as u8]);
    }

    #[test]
    fn test_4i() {
        let mut msg = String::new();
        for _ in 0..257 {
            msg.push('a');
        }
        let mut data: Vec<u8> = vec![];
        data.write_u32::<byteorder::BigEndian>((msg.len() + 4) as u32);
        data.extend_from_slice(msg.as_bytes());


        let mli: &dyn MLI = &MLI4I {};
        assert_eq!(mli.parse(&mut Cursor::new(data)).unwrap(), 257 as u32);
        assert_eq!(mli.create(&(msg.len() as usize)).unwrap(), vec![0x00, 0x00, 0x01 as u8, 0x05 as u8]);
    }
}