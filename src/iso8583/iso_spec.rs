use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::io::{BufReader, Cursor, Read};

use crate::iso8583::{bitmap, IsoError};
use crate::iso8583::bitmap::Bitmap;
use crate::iso8583::field::{Encoding, Field, FixedField, ParseError, VarField};
use crate::iso8583::yaml_de::{YMessageSegment, YField};

lazy_static! {
static ref ALL_SPECS: std::collections::HashMap<String,Spec> ={

        println!("current-dir: {}",std::env::current_dir().unwrap().to_str().unwrap());
        let mut spec_file = String::new();

        match std::env::var_os("SPEC_FILE") {
            Some(v) => {
            spec_file.push_str(v.to_str().unwrap());
            println!("spec-file: {}",spec_file)
            }
            None => panic!("SPEC_FILE env variable not defined!")
            }



    let mut specs=HashMap::<String,Spec>::new();

    match crate::iso8583::yaml_de::read_spec(spec_file.as_str()){
     Ok(spec)=>{
      specs.insert(String::from(spec.name()),spec);
     }
     Err(e)=> panic!(e.msg)
    };

    specs
};
}

// Spec is the definition of the spec - layout of fields etc..
pub struct Spec {
    pub(in crate::iso8583) name: String,
    pub(in crate::iso8583) id: u32,
    pub(in crate::iso8583) messages: Vec<MessageSegment>,
    pub(in crate::iso8583) header_fields: Vec<Box<dyn Field>>,
}


pub struct MessageSegment {
    pub(in crate::iso8583) name: String,
    pub(in crate::iso8583) id: u32,
    pub(in crate::iso8583) selector: Vec<String>,
    pub(in crate::iso8583) fields: Vec<Box<dyn Field>>,
}


impl From<&YMessageSegment> for MessageSegment {
    fn from(yms: &YMessageSegment) -> Self {
        let mut fields: Vec<Box<dyn Field>> = Vec::<Box<dyn Field>>::new();

        yms.fields.iter().for_each(|f| {
            fields.push(Into::<Box<dyn Field>>::into(f));
        });


        MessageSegment {
            name: yms.name.clone(),
            id: yms.id,
            selector: yms.selector.iter().map(|s| s.clone()).collect(),
            fields,
        }
    }
}


impl MessageSegment {
    pub fn name(&self) -> &str {
        return self.name.as_str();
    }

    pub fn field_by_name(&self, name: &String) -> Result<&dyn Field, IsoError> {
        match self.fields.iter().find(|field| -> bool{
            if field.name() == name {
                true
            } else {
                false
            }
        }) {
            None => {
                //try bitmap
                let bmp = self.field_by_name(&"bitmap".to_string()).unwrap();
                Ok(bmp.child_by_name(name))
            }
            Some(f) => {
                Ok(f.as_ref())
            }
        }
    }
}

impl Spec {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn get_message(&self, name: &str) -> Result<&MessageSegment, IsoError> {
        for msg in &self.messages {
            if msg.name() == name {
                return Ok(msg);
            }
        }
        return Err(IsoError { msg: format!("{} message not found", name) });
    }

    pub fn get_message_from_header(&self, header_val: &str) -> Result<&MessageSegment, IsoError> {
        for msg in &self.messages {
            if msg.selector.contains(&header_val.to_string()) {
                return Ok(msg);
            }
        }
        return Err(IsoError { msg: format!("message not found for header - {}", header_val) });
    }

    pub fn get_msg_segment(&'static self, data: &Vec<u8>) -> Result<&MessageSegment, IsoError> {
        let mut selector = String::new();
        let mut f2d_map = HashMap::new();

        let mut in_buf = Cursor::new(data);

        for f in &self.header_fields {
            match f.parse(&mut in_buf, &mut f2d_map) {
                Ok(_) => {
                    selector.extend(f.to_string(f2d_map.get(f.name()).unwrap()).chars());
                }
                Err(e) => {
                    return Err(IsoError { msg: e.msg });
                }
            }
        }

        debug!("computed header value for incoming message = {}", selector);
        match self.get_message_from_header(selector.as_str()) {
            Ok(msg) => {
                Ok(msg)
            }
            Err(e) => Err(e)
        }
    }
}

// IsoMsg represents a parsed message for a given spec
pub struct IsoMsg {
    pub spec: &'static Spec,
    pub msg: &'static MessageSegment,
    // field data map - name to raw value
    pub fd_map: std::collections::HashMap<String, Vec<u8>>,
    // the bitmap on the iso message
    pub bmp: bitmap::Bitmap,
}

impl IsoMsg {
    pub fn spec(&self) -> &'static Spec {
        self.spec
    }

    // Returns the value of a field by position in the bitmap
    pub fn bmp_child_value(&self, pos: u32) -> Result<String, IsoError> {
        let f = self.msg.fields.iter().find(|f| -> bool {
            if f.name() == "bitmap" {
                true
            } else {
                false
            }
        }).unwrap();

        let cf = f.child_by_pos(pos);
        match self.fd_map.get(cf.name()) {
            None => {
                Err(IsoError { msg: format!("no value for field at position {}", pos) })
            }
            Some(v) => {
                Ok(cf.to_string(v))
            }
        }
    }

    // Get the value of a top level field like message_type
    pub fn get_field_value(&self, name: &String) -> Result<String, IsoError> {
        match self.msg.fields.iter().find(|f| -> bool {
            if f.name() == name {
                true
            } else {
                false
            }
        }) {
            Some(f) => {
                Ok(f.to_string(self.fd_map.get(name).unwrap()))
            }
            None => {
                Err(IsoError { msg: format!("No such field : {}", name) })
            }
        }
    }

    // sets a top-level field like message_type etc
    pub fn set(&mut self, name: &str, val: &str) -> Result<(), IsoError> {
        match self.msg.field_by_name(&name.to_string()) {
            Ok(f) => {
                self.fd_map.insert(f.name().clone(), f.to_raw(val));
                Ok(())
            }
            Err(e) => Err(e)
        }
    }

    // Sets a field on the bitmap
    pub fn set_on(&mut self, pos: u32, val: &str) -> Result<(), IsoError> {
        match self.msg.field_by_name(&"bitmap".to_string()) {
            Ok(f) => {
                let cf = f.child_by_pos(pos);
                self.fd_map.insert(cf.name().clone(), cf.to_raw(val));
                self.bmp.set_on(pos);
                Ok(())
            }
            Err(e) => Err(e)
        }
    }

    pub fn echo_from(&mut self, req_msg: &IsoMsg, positions: &[u32]) -> Result<(), IsoError> {
        match self.msg.field_by_name(&"bitmap".to_string()) {
            Ok(f) => {
                for pos in positions {
                    let cf = f.child_by_pos(*pos);
                    match req_msg.bmp_child_value(*pos) {
                        Ok(res) => {
                            debug!("echoing .. {}: {}", pos, res);
                            self.fd_map.insert(cf.name().clone(), cf.to_raw(res.as_str()));
                            self.bmp.set_on(*pos);
                        }
                        Err(e) => {
                            return Err(e);
                        }
                    }
                }
                Ok(())
            }
            Err(e) => Err(e)
        }
    }

    pub fn assemble(&self) -> Result<Vec<u8>, IsoError> {
        let mut out_buf: Vec<u8> = Vec::new();
        for f in &self.msg.fields {
            match f.assemble(&mut out_buf, &self) {
                Ok(_) => {}
                Err(e) => {
                    return Err(IsoError { msg: e.msg });
                }
            }
        }
        Ok(out_buf)
    }
}


impl Display for IsoMsg {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        let mut res = "".to_string();
        for (f, v) in &(self.fd_map) {
            let field = self.msg.field_by_name(f).unwrap();
            res = res + format!("\n{:20.40}: {} ", f, field.to_string(v)).as_str();
        }
        f.write_str(&res).unwrap();
        Ok(())
    }
}


pub fn spec(name: &str) -> &'static Spec {
    //TODO:: handle case of multiple specs, for now just return the first
    ALL_SPECS.iter().find_map(|(k, v)| Some(v)).unwrap()
}

impl Spec {
    pub fn parse(&'static self, data: &mut Vec<u8>) -> Result<IsoMsg, ParseError> {
        let msg = self.get_msg_segment(data);
        if msg.is_err() {
            return Err(ParseError { msg: msg.err().unwrap().msg });
        }

        let mut iso_msg = IsoMsg {
            spec: &self,
            msg: &msg.unwrap(),
            fd_map: HashMap::new(),
            bmp: bitmap::new_bmp(0, 0, 0),
        };

        let mut cp_data = Cursor::new(data);

        for f in &iso_msg.msg.fields {
            debug!("parsing field : {}", f.name());
            let res = match f.parse(&mut cp_data, &mut iso_msg.fd_map) {
                Err(e) => Result::Err(e),
                Ok(r) => {
                    Ok(())
                }
            };

            if res.is_err() {
                return Result::Err(res.err().unwrap());
            }
        }

        /*let res = cp_data.bytes().count();
        if res > 0 {
            warn!("residual data : {}", hex::encode(cp_data.bytes()..collect::<Vec<u8>>()))
        }*/

        Ok(iso_msg)
    }
}