use crate::iso8583::field::{Field, ParseError, Encoding};
use byteorder::ByteOrder;

#[derive(Debug)]
pub struct Bitmap {
    p_bmp: u64,
    s_bmp: u64,
    t_bmp: u64,
}

impl Bitmap {
    pub fn is_on(self: &Bitmap, pos: u32) -> bool {
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

    pub fn hex_string(self: &Bitmap) -> String {
        format!("{:016.0x}{:016.0x}{:016.0x}", self.p_bmp, self.s_bmp, self.t_bmp)
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

    fn parse(&self, in_buf: &mut Vec<u8>, iso_msg: &mut crate::iso8583::iso_spec::IsoMsg) -> Result<u32, ParseError> {
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
            iso_msg.fd_map.insert(self.name.clone(), f_data);
            iso_msg.bmp = bmp;


            for i in 2..193 {
                if iso_msg.bmp.is_on(i) {
                    let is_present = self.by_position(i);
                    match match is_present {
                        Ok(f) => {
                            debug!("parsing field - {}", f.name());
                            match f.parse(in_buf, iso_msg) {
                                Ok(_) => Ok(0),
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

            Ok(0)
        } else {
            Err(ParseError { msg: format!("require {} but have {}", 8, in_buf.capacity()) })
        }
    }

    fn assemble(&self, _: &mut Vec<u8>) -> Result<u32, ParseError> {
        unimplemented!()
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
}