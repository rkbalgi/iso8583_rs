use crate::iso8583::field::{FixedField, VarField, Field, Encoding, ParseError};
use crate::iso8583::{bitmap, IsoError};


use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use crate::iso8583::server::IsoServerError;
use crate::iso8583::bitmap::Bitmap;


lazy_static! {
static ref ALL_SPECS: std::collections::HashMap<String,Spec> ={

    let mut specs=HashMap::new();

    //TODO:: externalize this into a spec (see sample_spec.yaml - WIP)
    specs.insert("SampleSpec".to_string(),Spec {
        name: "SampleSpec",
        header_fields:vec![
            Box::new(FixedField { name: "message_type".to_string(), len: 4, encoding: Encoding::ASCII ,position: 0}),
            ],
        messages: vec![
          MessageSegment{
                 selector: vec!["1100"],
                 name: "Authorization Request - 1100",
                 req_fields:  vec![
            Box::new(FixedField { name: "message_type".to_string(), len: 4, encoding: Encoding::ASCII ,position: 0}),
            Box::new(bitmap::BmpField { name: "bitmap".to_string(), encoding: Encoding::ASCII ,
                 children: vec![
                                Box::new(VarField { name: "pan".to_string(), len: 2, encoding: Encoding::ASCII, len_encoding: Encoding::ASCII, position:2 }),
                                Box::new(FixedField { name: "proc_code".to_string(), len: 6, encoding: Encoding::ASCII, position:3 }),
                                Box::new(FixedField { name: "amount".to_string(), len: 12, encoding: Encoding::ASCII, position:4 }),
                                Box::new(FixedField { name: "stan".to_string(), len: 6, encoding: Encoding::ASCII, position:11 }),
                                Box::new(FixedField { name: "expiration_date".to_string(), len: 4, encoding: Encoding::ASCII, position: 14 }),
                               ]}),

        ]} /*end auth 1100 message*/,
        MessageSegment{
                 selector: vec!["1110"],
                 name: "Authorization Response - 1110",
                 req_fields:  vec![
            Box::new(FixedField { name: "message_type".to_string(), len: 4, encoding: Encoding::ASCII ,position: 0}),
            Box::new(bitmap::BmpField { name: "bitmap".to_string(), encoding: Encoding::ASCII ,
                 children: vec![
                                Box::new(VarField { name: "pan".to_string(), len: 2, encoding: Encoding::ASCII, len_encoding: Encoding::ASCII, position:2 }),
                                Box::new(FixedField { name: "proc_code".to_string(), len: 6, encoding: Encoding::ASCII, position:3 }),
                                Box::new(FixedField { name: "amount".to_string(), len: 12, encoding: Encoding::ASCII, position:4 }),
                                Box::new(FixedField { name: "stan".to_string(), len: 6, encoding: Encoding::ASCII, position:11 }),
                                Box::new(FixedField { name: "expiration_date".to_string(), len: 4, encoding: Encoding::ASCII, position: 14 }),
                                Box::new(FixedField { name: "approval_code".to_string(), len: 6, encoding: Encoding::ASCII, position:38 }),
                                Box::new(FixedField { name: "action_code".to_string(), len: 3, encoding: Encoding::ASCII, position:39 }),
                               ]}),

        ]} /*end auth 1110 message*/,
          ] /*end messages*/,


    });

    specs
};
}

// Spec is the definition of the spec - layout of fields etc..
pub struct Spec {
    name: &'static str,
    messages: Vec<MessageSegment>,
    header_fields: Vec<Box<dyn Field>>,
}

pub struct MessageSegment {
    name: &'static str,
    selector: Vec<&'static str>,
    req_fields: Vec<Box<dyn Field>>,
}


impl MessageSegment {
    pub fn name(&self) -> &str {
        return self.name;
    }

    pub fn field_by_name(&self, name: &String) -> Result<&dyn Field, IsoError> {
        match self.req_fields.iter().find(|field| -> bool{
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
            if msg.selector.contains(&header_val) {
                return Ok(msg);
            }
        }
        return Err(IsoError { msg: format!("message not found for header - {}", header_val) });
    }

    pub fn get_msg_segment(&'static self, data: &mut Vec<u8>) -> Result<&MessageSegment, IsoError> {
        let mut selector = String::new();
        let mut f2d_map = HashMap::new();
        for f in &self.header_fields {
            match f.parse(data, &mut f2d_map) {
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
        let f = self.msg.req_fields.iter().find(|f| -> bool {
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
        match self.msg.req_fields.iter().find(|f| -> bool {
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
        for f in &self.msg.req_fields {
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
    return ALL_SPECS.get(name).unwrap();
}

impl Spec {
    pub fn parse(&'static self, data: Vec<u8>) -> Result<IsoMsg, ParseError> {
        let mut cp_data = data.clone();

        let msg = self.get_msg_segment(&mut data.clone());
        if msg.is_err() {
            return Err(ParseError { msg: msg.err().unwrap().msg });
        }

        let mut iso_msg = IsoMsg {
            spec: &self,
            msg: &msg.unwrap(),
            fd_map: HashMap::new(),
            bmp: bitmap::new_bmp(0, 0, 0),
        };

        for f in &iso_msg.msg.req_fields {
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

        if cp_data.len() > 0 {
            warn!("residual data : {}", hex::encode(cp_data.iter()))
        }

        Ok(iso_msg)
    }
}