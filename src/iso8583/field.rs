use crate::iso8583::bitmap::Bitmap;
use std::error::Error;
use std::fmt;

pub enum Encoding {
    ASCII,
    EBCDIC,
    BINARY,
    BCD,
}

#[derive(Debug)]
pub struct ParseError {
    msg: String
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(iso8583:: parse-error: {})", self.msg)
    }
}


pub trait Field:Sync {
    fn name(&self) -> &String;
    fn parse(&self, in_buf: &mut Vec<u8>) -> Result<u32, ParseError>;
    fn assemble(&self, out_buf: &mut Vec<u8>) -> Result<u32, ParseError>;
}

pub struct FixedField {
    pub name: String,
    pub len: u32,
    pub encoding: Encoding,
}

impl Field for FixedField {
    fn name(&self) -> &String {
        &self.name
    }

    fn parse(self: &Self, in_buf: &mut Vec<u8>) -> Result<u32, ParseError> {
        if self.len < in_buf.capacity() as u32 {

            let mut f_data= Vec::new();
            for i in 0..self.len {
                f_data.push(in_buf.remove(0))
            }

            for i in f_data.iter() {
                print!("{:?}", i);
            }
            println!("{:?}", " done ....");
            Ok(0)
        } else {
            Result::Err(ParseError { msg: format!("require {} but have {}", self.len, in_buf.capacity()) })
        }
    }

    fn assemble(self: &Self, out_buf: &mut Vec<u8>) -> Result<u32, ParseError> {
        unimplemented!()
    }
}

struct BmpField {
    name: String,
    encoding: Encoding,
}

impl Field for BmpField {
    fn name(&self) -> &String {
        &self.name
    }

    fn parse(&self, _: &mut Vec<u8>) -> Result<u32, ParseError> {
        unimplemented!()
    }

    fn assemble(&self, _: &mut Vec<u8>) -> Result<u32, ParseError> {
        unimplemented!()
    }
}

struct VarField {
    //number of bytes in the length indicator
    name: String,
    len: u32,
    len_encoding: Encoding,
    encoding: Encoding,
}

impl Field for VarField
{
    fn name(&self) -> &String {
        &self.name
    }

    fn parse(&self, _: &mut Vec<u8>) -> Result<u32, ParseError> {
        unimplemented!()
    }

    fn assemble(&self, _: &mut Vec<u8>) -> Result<u32, ParseError> {
        unimplemented!()
    }
}


