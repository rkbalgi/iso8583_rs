use crate::iso8583::iso_spec::IsoMsg;
use std::fmt;
use crate::iso8583::field::Encoding::{ASCII, EBCDIC, BCD, BINARY};
use std::collections::HashMap;
use std::io::{BufReader, BufRead, Error};

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
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
    fn parse(&self, in_buf: &mut dyn BufRead, f2d_map: &mut HashMap<String, Vec<u8>>) -> Result<(), ParseError>;
    fn assemble(&self, out_buf: &mut Vec<u8>, iso_msg: &IsoMsg) -> Result<u32, ParseError>;

    fn position(&self) -> u32;
    fn children(&self) -> Vec<&dyn Field>;
    fn child_by_pos(&self, pos: u32) -> &dyn Field;
    fn child_by_name(&self, name: &String) -> &dyn Field;
    fn to_string(&self, data: &Vec<u8>) -> String;
    fn to_raw(&self, val: &str) -> Vec<u8>;
}

pub struct FixedField {
    pub name: String,
    pub id: u32,
    pub len: u32,
    pub encoding: Encoding,
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


pub struct VarField {
    pub name: String,
    pub id: u32,
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
            _ => unimplemented!("only ascii supported for length encoding on var fields")
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
            let mut ascii_str = String::new();
            data.iter().for_each(|f| ascii_str.push(char::from(encoding8::ebcdic::to_ascii(f.clone()))));
            ascii_str
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





