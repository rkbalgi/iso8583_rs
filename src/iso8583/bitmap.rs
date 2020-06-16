use crate::iso8583::field::{Field, ParseError, Encoding};
use byteorder::ByteOrder;
use crate::iso8583::iso_spec;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Bitmap {
    p_bmp: u64,
    s_bmp: u64,
    t_bmp: u64,
}

impl Bitmap {
    pub fn is_on(&self, pos: u32) -> bool {
        assert!(pos > 0 && pos <= 192);
        //println!("{:0x}{}", self.p_bmp >> 8, self.p_bmp >> ((64 as u32) - pos) as u64);

        if pos < 65 {
            self.p_bmp >> ((64 as u32) - pos) as u64 & 0x01 == 0x01
        } else if pos > 64 && pos < 129 {
            self.s_bmp >> ((64 as u32) - (pos - 64)) as u64 & 0x01 == 0x01
        } else {
            self.t_bmp >> ((64 as u32) - (pos - 128)) as u64 & 0x01 == 0x01
        }
    }

    pub fn set_on(&mut self, pos: u32) {
        assert!(pos > 0 && pos <= 192);

        if pos < 65 {
            self.p_bmp = ((0x8000000000000000 as u64) >> (pos - 1) as u64) | self.p_bmp;
        } else if pos > 64 && pos < 129 {
            self.s_bmp = ((0x8000000000000000 as u64) >> (pos - 64 - 1) as u64) | self.s_bmp;
            if !self.is_on(1) {
                self.set_on(1);
            }
        } else {
            self.t_bmp = ((0x8000000000000000 as u64) >> (pos - 128 - 1) as u64) | self.t_bmp;
            if !self.is_on(65) {
                self.set_on(65);
            }
        }
    }

    pub fn hex_string(&self) -> String {
        format!("{:016.0x}{:016.0x}{:016.0x}", self.p_bmp, self.s_bmp, self.t_bmp)
    }

    pub fn as_vec(&self) -> Vec<u8> {
        let mut bmp_data = vec![0; 8];

        byteorder::BigEndian::write_u64(&mut bmp_data[0..], self.p_bmp);
        if ((self.p_bmp >> 63) & 0x01) == 0x01 {
            bmp_data.resize(16, 0);
            byteorder::BigEndian::write_u64(&mut bmp_data[8..], self.s_bmp);
        }
        if ((self.s_bmp >> 63) & 0x01) == 0x01 {
            bmp_data.resize(24, 0);
            byteorder::BigEndian::write_u64(&mut bmp_data[16..], self.t_bmp);
        }

        bmp_data
    }
}

#[test]
fn test_bmp() {
    let mut bmp = new_bmp(0, 0, 0);
    bmp.set_on(4);
    bmp.set_on(11);
    bmp.set_on(64);
    bmp.set_on(99);
    bmp.set_on(133);
    bmp.set_on(6);

    for i in 1..193 {
        if bmp.is_on(i) {
            println!("{} is on ", i)
        }
    }
}


pub fn new_bmp(b1: u64, b2: u64, b3: u64) -> Bitmap {
    Bitmap {
        p_bmp: b1,
        s_bmp: b2,
        t_bmp: b3,
    }
}


pub struct BmpField {
    pub name: String,
    pub encoding: Encoding,
    pub children: Vec<Box<dyn Field>>,
}


impl BmpField {
    pub fn by_position(&self, pos: u32) -> Result<&Box<dyn Field>, ParseError> {
        let opt = &(self.children).iter().filter(|f| -> bool{
            if f.as_ref().position() == pos {
                true
            } else {
                false
            }
        }).next();

        match opt {
            Some(f) => Ok(f),
            None => Err(ParseError { msg: format!("position {} not defined", pos) }),
        }
    }
}

impl Field for BmpField {
    fn name(&self) -> &String {
        &self.name
    }

    fn parse(&self, in_buf: &mut Vec<u8>, f2d_map: &mut HashMap<String, Vec<u8>>) -> Result<(), ParseError> {
        if in_buf.capacity() as u32 >= 8 {
            let mut f_data = Vec::new();
            for _ in 0..8 {
                f_data.push(in_buf.remove(0));
            }
            trace!("parsed-data: {} := {}", self.name, hex::encode(f_data.iter()));


            let b1 = byteorder::BigEndian::read_u64(f_data.as_slice());

            //TODO:: support secondary and tertiary bitmaps
            if b1 & 0x80 == 0x80 {
                unimplemented!("include support for secondary/tertiary bitmaps...");
            }

            let bmp = new_bmp(b1, 0, 0);
            //iso_msg.bmp = bmp;
            f2d_map.insert(self.name().to_string(), bmp.as_vec());


            for i in 2..193 {
                if bmp.is_on(i) {
                    let is_present = self.by_position(i);
                    match match is_present {
                        Ok(f) => {
                            debug!("parsing field - {}", f.name());
                            match f.parse(in_buf, f2d_map) {
                                Ok(r) => {
                                    Ok(())
                                }
                                Err(e) => Err(e),
                            }
                        }
                        Err(e) => Err(e),
                    }
                    {
                        Err(e) => {
                            return Err(e);
                        }
                        _ => {}
                    }
                }
            }

            Ok(())
        } else {
            Err(ParseError { msg: format!("require {} but have {}", 8, in_buf.capacity()) })
        }
    }

    fn assemble(&self, out_buf: &mut Vec<u8>, iso_msg: &iso_spec::IsoMsg) -> Result<u32, ParseError> {
        let bmp_data = iso_msg.bmp.as_vec();
        out_buf.extend(bmp_data);

        for pos in 2..193 {
            if iso_msg.bmp.is_on(pos) {
                match self.by_position(pos) {
                    Ok(f) => {
                        match iso_msg.fd_map.get(f.name()) {
                            Some(fd) => {
                                f.assemble(out_buf, iso_msg);
                            }
                            None => { return Err(ParseError { msg: format!("position {} is on, but no field data present!", pos) }); }
                        };
                    }
                    Err(e) => return Err(e)
                }
            }
        };

        Ok(0)
    }


    fn position(&self) -> u32 {
        0
    }

    fn child_by_pos(&self, pos: u32) -> &dyn Field {
        self.children.iter().find(|f| -> bool {
            if f.position() == pos {
                true
            } else {
                false
            }
        }).unwrap().as_ref()
    }

    fn child_by_name(&self, name: &String) -> &dyn Field {
        self.children.iter().find(|f| -> bool {
            if f.name() == name {
                true
            } else {
                false
            }
        }).unwrap().as_ref()
    }

    fn to_string(&self, data: &Vec<u8>) -> String {
        hex::encode(data)
    }

    fn to_raw(&self, val: &str) -> Vec<u8> {
        unimplemented!()
    }
}