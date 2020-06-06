use crate::iso8583::iso_spec::IsoMsg;
use std::fmt;

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
        println!("before_parse:: {}", hex::encode(in_buf.as_slice()));
        if self.len < in_buf.capacity() as u32 {
            let mut f_data = Vec::new();
            for _ in 0..self.len {
                f_data.push(in_buf.remove(0));
            }
            println!("parsed-data: {}", hex::encode(f_data.as_slice()));
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



pub struct VarField {
    pub name: String,
    //number of bytes in the length indicator
    pub len: u32,
    pub len_encoding: Encoding,
    pub encoding: Encoding,
    pub position: u32,
}


impl VarField {
    fn data_len(&self, data: &Vec<u8>) -> usize
    {
        match self.len_encoding {
            Encoding::ASCII => {
                String::from_utf8(data.clone()).expect("").parse::<usize>().unwrap()
            }
            _ => unimplemented!("only ascii supported for length encoding on var fields"),
        }
    }
}

impl Field for VarField
{
    fn name(&self) -> &String {
        &self.name
    }

    fn parse(&self, in_buf: &mut Vec<u8>, iso_msg: &mut IsoMsg) -> Result<u32, ParseError> {
        println!("before_parse:: {}", hex::encode(in_buf.as_slice()));
        if self.len < in_buf.capacity() as u32 {
            let mut len_data = Vec::with_capacity(self.len as usize);

            for _ in 0..self.len {
                (len_data).push(in_buf.remove(0));
            }
            println!("parsed-data (len-ind) : {}", hex::encode(&len_data));


            let data_len = self.data_len(&len_data);
            let mut f_data = Vec::with_capacity(data_len as usize);
            for _ in 0..data_len {
                (f_data).push(in_buf.remove(0));
            }
            iso_msg.fd_map.insert(self.name.clone(), f_data);


            Ok(0)
        } else {
            Result::Err(ParseError { msg: format!("require {} but have {}", self.len, in_buf.capacity()) })
        }
    }


    fn assemble(&self, _: &mut Vec<u8>) -> Result<u32, ParseError> {
        unimplemented!()
    }

    fn position(&self) -> u32 {
        return self.position;
    }
}





