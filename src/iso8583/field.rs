use crate::iso8583::iso_spec::IsoMsg;
use std::fmt;
use crate::iso8583::field::Encoding::ASCII;
use std::collections::HashMap;

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
    fn parse(&self, in_buf: &mut Vec<u8>, f2d_map: &mut HashMap<String, Vec<u8>>) -> Result<(), ParseError>;
    fn assemble(&self, out_buf: &mut Vec<u8>, iso_msg: &IsoMsg) -> Result<u32, ParseError>;

    fn position(&self) -> u32;
    fn child_by_pos(&self, pos: u32) -> &dyn Field;
    fn child_by_name(&self, name: &String) -> &dyn Field;
    fn to_string(&self, data: &Vec<u8>) -> String;
    fn to_raw(&self, val: &str) -> Vec<u8>;
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

    fn parse(self: &Self, in_buf: &mut Vec<u8>, f2d_map: &mut HashMap<String, Vec<u8>>) -> Result<(), ParseError> {
        trace!("buf before_parse:: {}", hex::encode(in_buf.as_slice()));
        if self.len < in_buf.capacity() as u32 {
            let mut f_data = Vec::new();
            for _ in 0..self.len {
                f_data.push(in_buf.remove(0));
            }
            trace!("parsed-data: {}", hex::encode(f_data.as_slice()));
            f2d_map.insert(self.name.clone(), f_data);

            Ok(())
        } else {
            Err(ParseError { msg: format!("require {} but have {}", self.len, in_buf.capacity()) })
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

    fn parse(&self, in_buf: &mut Vec<u8>, f2d_map: &mut HashMap<String, Vec<u8>>) -> Result<(), ParseError> {
        trace!("buf before_parse:: {}", hex::encode(in_buf.as_slice()));

        if self.len < in_buf.capacity() as u32 {
            let mut len_data = Vec::with_capacity(self.len as usize);

            for _ in 0..self.len {
                (len_data).push(in_buf.remove(0));
            }
            trace!("parsed-data (len-ind) : {}", hex::encode(&len_data));


            let data_len = self.data_len(&len_data);
            let mut f_data = Vec::with_capacity(data_len as usize);
            for _ in 0..data_len {
                (f_data).push(in_buf.remove(0));
            }

            f2d_map.insert(self.name.clone(), f_data);

            Ok(())
        } else {
            Result::Err(ParseError { msg: format!("require {} but have {}", self.len, in_buf.capacity()) })
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

fn vec_to_string(encoding: &Encoding, data: &Vec<u8>) -> String {
    match encoding {
        ASCII => {
            String::from_utf8(data.clone()).unwrap()
        }
        _ => {
            hex::encode(data.as_slice())
        }
    }
}


fn string_to_vec(encoding: &Encoding, data: &str) -> Vec<u8> {
    match encoding {
        ASCII => {
            data.to_string().into_bytes()
        }
        _ => {
            hex::decode(data).unwrap()
        }
    }
}





