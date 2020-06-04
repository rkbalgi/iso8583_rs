use crate::iso8583::bitmap::new_bmp;
use crate::iso8583::bitmap::Bitmap;
use crate::iso8583::iso_spec::IsoMsg;
use std::error::Error;
use std::fmt;
use byteorder;
use byteorder::ByteOrder;


pub enum Encoding {
    ASCII,
    EBCDIC,
    BINARY,
    BCD,
}


#[derive(Debug)]
pub struct ParseError {
    pub msg: String
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(iso8583:: parse-error: {})", self.msg)
    }
}


pub trait Field: Sync {
    fn name(&self) -> &String;
    fn parse(&self, in_buf: &mut Vec<u8>, iso_msg: &mut IsoMsg) -> Result<u32, ParseError>;
    fn assemble(&self, out_buf: &mut Vec<u8>) -> Result<u32, ParseError>;

    fn position(&self) -> u32;
}

pub struct FixedField {
    pub name: String,
    pub len: u32,
    pub encoding: Encoding,
    pub position: u32,
}

impl Field for FixedField {
    fn name(&self) -> &String {
        &self.name
    }

    fn parse(self: &Self, in_buf: &mut Vec<u8>, iso_msg: &mut IsoMsg) -> Result<u32, ParseError> {
        if self.len < in_buf.capacity() as u32 {
            let mut f_data = Vec::new();
            for i in 0..self.len {
                f_data.push(in_buf.pop().expect(""))
            }
            println!("parsed-data: {}", hex::encode(f_data.iter()));
            iso_msg.fd_map.insert(self.name.clone(), f_data);

            Ok(0)
        } else {
            Result::Err(ParseError { msg: format!("require {} but have {}", self.len, in_buf.capacity()) })
        }
    }

    fn assemble(self: &Self, out_buf: &mut Vec<u8>) -> Result<u32, ParseError> {
        unimplemented!()
    }

    fn position(&self) -> u32 {
        return self.position;
    }
}

pub struct BmpField {
    pub name: String,
    pub encoding: Encoding,
}

impl Field for BmpField {
    fn name(&self) -> &String {
        &self.name
    }

    fn parse(&self, in_buf: &mut Vec<u8>, iso_msg: &mut IsoMsg) -> Result<u32, ParseError> {
        if in_buf.capacity() as u32 >= 8 {
            let mut f_data = Vec::new();
            for i in 0..8 {
                f_data.push(in_buf.pop().expect(""))
            }
            println!("parsed-data: {} := {}", self.name, hex::encode(f_data.iter()));


            let b1 = byteorder::BigEndian::read_u64(f_data.as_slice());


            let bmp = new_bmp(b1, 0, 0);
            iso_msg.fd_map.insert(self.name.clone(), f_data);
            iso_msg.bmp = bmp;

            for i in 2..193 {
                if iso_msg.bmp.is_on(i) {
                    return match iso_msg.spec.by_position(i) {
                        Ok(f) => match f.parse(in_buf, iso_msg) {
                            Ok(_) => Ok(0),
                            Err(e) => Err(e),
                        }
                        Err(e) => Err(e),
                    };
                }
            }

            Ok(0)
        } else {
            Err(ParseError { msg: format!("require {} but have {}", 8, in_buf.capacity()) })
        }
    }

    fn assemble(&self, _: &mut Vec<u8>) -> Result<u32, ParseError> {
        unimplemented!()
    }

    fn position(&self) -> u32 {
        unimplemented!()
    }
}

pub struct VarField {
    pub name: String,
    //number of bytes in the length indicator
    pub len: u32,
    pub len_encoding: Encoding,
    pub encoding: Encoding,
    pub position: u32,
}

impl Field for VarField
{
    fn name(&self) -> &String {
        &self.name
    }

    fn parse(&self, _: &mut Vec<u8>, _: &mut IsoMsg) -> Result<u32, ParseError> {
        unimplemented!()
    }

    fn assemble(&self, _: &mut Vec<u8>) -> Result<u32, ParseError> {
        unimplemented!()
    }

    fn position(&self) -> u32 {
        return self.position;
    }
}






