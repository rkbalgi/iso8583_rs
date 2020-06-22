//! This module contains implementation of Variable and Fixed fields
//!
use crate::iso8583::iso_spec::IsoMsg;
use std::fmt;
use crate::iso8583::field::Encoding::{ASCII, EBCDIC, BCD, BINARY};
use std::collections::HashMap;
use std::io::{BufReader, BufRead, Error};

use serde::{Serialize, Deserialize};

/// This enum represents the encoding of a field (or length indicator for variable fields)
#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub enum Encoding {
    ASCII,
    EBCDIC,
    BINARY,
    BCD,
}

/// This struct represents a error in parsing a field/message
#[derive(Debug)]
pub struct ParseError {
    pub msg: String
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(iso8583:: parse-error: {})", self.msg)
    }
}

/// This trait represents a ISO field (specific implementations are FixedField, VarField and BmpField)
pub trait Field: Sync {
    /// Returns the name of the field
    fn name(&self) -> &String;

    /// Parses the field by reading from in_buf and stores the result into f2d_map
    /// Returns a ParseError on failure
    fn parse(&self, in_buf: &mut dyn BufRead, f2d_map: &mut HashMap<String, Vec<u8>>) -> Result<(), ParseError>;

    /// Assembles the field i.e. appends it data into out_buf
    /// Returns the number of bytes written on success or a ParseError on failure
    fn assemble(&self, out_buf: &mut Vec<u8>, iso_msg: &IsoMsg) -> Result<u32, ParseError>;

    /// Returns the position of the field in the parent field (mostly applicable for chlidren of BmpField)
    fn position(&self) -> u32;

    /// Returns children as Vec
    fn children(&self) -> Vec<&dyn Field>;

    /// Returns the child field by position
    fn child_by_pos(&self, pos: u32) -> &dyn Field;

    /// Returns child field by name
    fn child_by_name(&self, name: &String) -> &dyn Field;

    /// Returns a string that represents the field value in ascii
    fn to_string(&self, data: &Vec<u8>) -> String;

    /// Returns field value as binary (wire format)
    fn to_raw(&self, val: &str) -> Vec<u8>;
}

/// This struct represents a Fixed field
pub struct FixedField {
    /// Name of the field
    pub name: String,
    /// ID of the field (unused)
    pub id: u32,
    // Fixed length of the field
    pub len: u32,
    // Encoding of the field content
    pub encoding: Encoding,
    // Position of the field within the parent
    pub position: u32,
}

impl Field for FixedField {
    fn name(&self) -> &String {
        &self.name
    }

    fn parse(self: &Self, in_buf: &mut dyn BufRead, f2d_map: &mut HashMap<String, Vec<u8>>) -> Result<(), ParseError> {
        let mut f_data = vec![0; self.len as usize];
        match in_buf.read_exact(&mut f_data[..]) {
            Ok(_) => {
                f2d_map.insert(self.name.clone(), f_data);
                Ok(())
            }
            Err(_) => {
                Err(ParseError { msg: format!("not enough data to parse - {}", self.name) })
            }
        }
    }

    fn assemble(self: &Self, out_buf: &mut Vec<u8>, iso_msg: &IsoMsg) -> Result<u32, ParseError> {
        match iso_msg.fd_map.get(&self.name) {
            Some(fd) => {
                out_buf.extend(fd);
                Ok(fd.as_slice().len() as u32)
            }
            None => {
                Err(ParseError { msg: format!("field {} is not available!", self.name) })
            }
        }
    }

    fn position(&self) -> u32 {
        return self.position;
    }

    fn children(&self) -> Vec<&dyn Field> {
        //TODO:: when we choose to implement nested fields
        vec![]
    }

    fn child_by_pos(&self, pos: u32) -> &dyn Field {
        unimplemented!()
    }

    fn child_by_name(&self, name: &String) -> &dyn Field {
        unimplemented!()
    }

    fn to_string(&self, data: &Vec<u8>) -> String {
        vec_to_string(&self.encoding, data)
    }

    fn to_raw(&self, val: &str) -> Vec<u8> {
        string_to_vec(&self.encoding, val)
    }
}

/// This struct represents a Variable field
pub struct VarField {
    // Name of the field
    pub name: String,
    pub id: u32,
    /// Number of bytes in the length indicator
    pub len: u32,
    /// Encoding of the length indicator
    pub len_encoding: Encoding,
    /// Encoding of field content
    pub encoding: Encoding,
    // Position of field within parent
    pub position: u32,
}


impl VarField {
    /// Returns the length of data in the variable field
    fn data_len(&self, data: &Vec<u8>) -> usize
    {
        match self.len_encoding {
            Encoding::ASCII => {
                String::from_utf8(data.clone()).expect("").parse::<usize>().unwrap()
            }
            Encoding::EBCDIC => {
                ebcdic_to_ascii(data).parse::<usize>().unwrap()
            }
            _ => unimplemented!("only ascii supported for length encoding on var fields"),
        }
    }

    /// Builds and returns the length indicator based on encoding of the field as a Vec<u8>
    fn build_len_ind(&self, len: usize) -> Vec<u8> {
        match self.len_encoding {
            Encoding::ASCII => {
                match self.len {
                    1 => format!("{:01}", len).into_bytes(),
                    2 => format!("{:02}", len).into_bytes(),
                    3 => format!("{:03}", len).into_bytes(),
                    _ => unimplemented!("len-ind cannot exceed 3")
                }
            }
            Encoding::EBCDIC => {
                match self.len {
                    1 => ascii_to_ebcdic(&mut format!("{:01}", len).into_bytes()),
                    2 => ascii_to_ebcdic(&mut format!("{:02}", len).into_bytes()),
                    3 => ascii_to_ebcdic(&mut format!("{:03}", len).into_bytes()),
                    _ => unimplemented!("len-ind cannot exceed 3")
                }
            }
            _ => unimplemented!("only ascii supported for length encoding on var fields - {:?}", self.len_encoding)
        }
    }
}

impl Field for VarField
{
    fn name(&self) -> &String {
        &self.name
    }

    fn parse(&self, in_buf: &mut dyn BufRead, f2d_map: &mut HashMap<String, Vec<u8>>) -> Result<(), ParseError> {
        let mut len_data = vec![0; self.len as usize];
        match in_buf.read_exact(&mut len_data[..]) {
            Ok(_) => {
                trace!("parsed-data (len-ind) : {}", hex::encode(&len_data));


                let data_len = self.data_len(&len_data);
                let mut f_data = vec![0; data_len as usize];

                match in_buf.read_exact(&mut f_data[..]) {
                    Ok(_) => {
                        f2d_map.insert(self.name.clone(), f_data);
                        Ok(())
                    }
                    Err(e) => {
                        Result::Err(ParseError { msg: format!("insufficient data, failed to parse {}", self.name) })
                    }
                }
            }
            Err(_) => {
                Result::Err(ParseError { msg: format!("insufficient data, failed to parse length indicator for -  {}", self.name) })
            }
        }
    }


    fn assemble(&self, out_buf: &mut Vec<u8>, iso_msg: &IsoMsg) -> Result<u32, ParseError> {
        match iso_msg.fd_map.get(&self.name) {
            Some(fd) => {
                let len_ind = self.build_len_ind(fd.len());
                out_buf.extend(len_ind);
                out_buf.extend(fd);
                //fd.as_slice().iter().for_each(|d| out_buf.push(*d));
                Ok(fd.as_slice().len() as u32)
            }
            None => {
                Err(ParseError { msg: format!("field {} is not available!", self.name) })
            }
        }
    }


    fn position(&self) -> u32 {
        return self.position;
    }


    fn children(&self) -> Vec<&dyn Field> {
        //TODO:: when we choose to implement nested fields
        vec![]
    }


    fn child_by_pos(&self, pos: u32) -> &dyn Field {
        unimplemented!()
    }

    fn child_by_name(&self, name: &String) -> &dyn Field {
        unimplemented!()
    }

    fn to_string(&self, data: &Vec<u8>) -> String {
        vec_to_string(&self.encoding, data)
    }

    fn to_raw(&self, val: &str) -> Vec<u8> {
        string_to_vec(&self.encoding, val)
    }
}

pub(in crate::iso8583) fn vec_to_string(encoding: &Encoding, data: &Vec<u8>) -> String {
    match encoding {
        ASCII => {
            String::from_utf8(data.clone()).unwrap()
        }
        EBCDIC => {
            ebcdic_to_ascii(data)
        }
        BINARY => {
            hex::encode(data.as_slice())
        }
        BCD => {
            hex::encode(data.as_slice())
        }
        _ => panic!("unsupported encoding - {:?}", encoding)
    }
}

fn ebcdic_to_ascii(data: &Vec<u8>) -> String {
    let mut ascii_str = String::new();
    data.iter().for_each(|f| ascii_str.push(char::from(encoding8::ebcdic::to_ascii(f.clone()))));
    ascii_str
}

fn ascii_to_ebcdic(data: &mut Vec<u8>) -> Vec<u8> {
    for i in 0..data.len() {
        encoding8::ascii::make_ebcdic(data.get_mut(i).unwrap())
    }
    data.to_vec()
}


pub(in crate::iso8583) fn string_to_vec(encoding: &Encoding, data: &str) -> Vec<u8> {
    match encoding {
        ASCII => {
            data.to_string().into_bytes()
        }
        EBCDIC => {
            let mut ebcdic = vec![];
            (&mut data.to_string()).as_bytes().iter().for_each(|b| ebcdic.push(encoding8::ascii::to_ebcdic(b.clone())));
            ebcdic
        }
        BINARY => {
            hex::decode(data).unwrap()
        }
        BCD => {
            hex::decode(data).unwrap()
        }
        _ => panic!("unsupported encoding - {:?}", encoding)
    }
}





