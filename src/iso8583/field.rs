use crate::iso8583::bitmap::Bitmap;
use std::error::Error;

pub(crate) enum Encoding {
    ASCII,
    EBCDIC,
    BINARY,
    BCD,
}


pub trait Field: Sized {
    fn parse(&self, in_buf: &Vec<u8>) -> Result<u32, dyn Error>;
    fn assemble(&self, out_buf: &mut Vec<u8>) -> Result<u32, dyn Error>;
}

pub struct FixedField {
    pub name: String,
    pub len: u32,
    pub encoding: Encoding,
}

impl Field for FixedField {
    fn parse(&self, in_buf: &Vec<u8>) -> Result<u32, dyn Error> {
        if in_buf.capacity() < self.len as usize {
            print!("{}", in_buf.as_slice()[0..self.len]);
            Ok(0)
        } else {
            Err(format!("require {} but have {}", self.len, in_buf.capacity()))
        }
    }

    fn assemble(&self, out_buf: &mut Vec<u8>) -> Result<u32, dyn Error> {
        unimplemented!()
    }
}

struct BmpField {
    encoding: Encoding,
}

impl Field for BmpField {
    fn parse(&self, _: Vec<u8>) -> Result<u32, dyn Error> {
        unimplemented!()
    }

    fn assemble(&self, _: &mut Vec<u8>) -> Result<u32, dyn Error> {
        unimplemented!()
    }
}

struct VarField {
    //number of bytes in the length indicator
    len: u32,
    len_encoding: Encoding,
    encoding: Encoding,
}

impl Field for VarField {
    fn parse(&self, _: Vec<u8>) -> Result<u32, dyn Error> {
        unimplemented!()
    }

    fn assemble(&self, _: &mut Vec<u8>) -> Result<u32, dyn Error> {
        unimplemented!()
    }
}


