use crate::iso8583::field::{FixedField, VarField, Field, Encoding, ParseError};
use crate::iso8583::{bitmap, IsoError};


use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use crate::iso8583::server::IsoServerError;


lazy_static! {
static ref ALL_SPECS: std::collections::HashMap<String,Spec> ={

    let mut specs=HashMap::new();

    specs.insert("SampleSpec".to_string(),Spec {
        name: "SampleSpec".to_string(),
        fields: vec![
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

        ],
    });

    specs
};
}

// Spec is the definition of the spec - layout of fields etc..
pub struct Spec {
    name: String,
    fields: Vec<Box<dyn Field>>,
}

impl Spec {
    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn field_by_name(&self, name: &String) -> Result<&dyn Field, IsoError> {
        match self.fields().iter().find(|field| -> bool{
            if field.name() == name {
                true
            } else {
                false
            }
        }) {
            None => {
                //try bitmap
                let bmp = self.field_by_name(&"bitmap".to_string()).unwrap();
                //https://stackoverflow.com/questions/33687447/how-to-get-a-reference-to-a-concrete-type-from-a-trait-object
                Ok(bmp.child_by_name(name))
            }
            Some(f) => {
                Ok(f.as_ref())
            }
        }
    }
}

// IsoMsg represents a parsed message for a given spec
pub struct IsoMsg {
    pub spec: &'static Spec,
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
        let f = self.spec.fields.iter().find(|f| -> bool {
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
        match self.spec.fields.iter().find(|f| -> bool {
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
        match self.spec.field_by_name(&name.to_string()) {
            Ok(f) => {
                self.fd_map.insert(f.name().clone(), f.to_raw(val));
                Ok(())
            }
            Err(e) => Err(e)
        }
    }

    // Sets a field on the bitmap
    pub fn set_on(&mut self, pos: u32, val: &str) -> Result<(), IsoError> {
        match self.spec.field_by_name(&"bitmap".to_string()) {
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
        match self.spec.field_by_name(&"bitmap".to_string()) {
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
        for f in &self.spec.fields {
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
            let field = self.spec.field_by_name(f).unwrap();
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
    pub fn fields(&self) -> &Vec<Box<dyn Field>> {
        &self.fields
    }
    pub fn parse(&'static self, data: Vec<u8>) -> Result<IsoMsg, ParseError> {
        let mut cp_data = data.clone();
        let mut iso_msg = IsoMsg { spec: &self, fd_map: HashMap::new(), bmp: bitmap::new_bmp(0, 0, 0) };

        for f in self.fields() {
            debug!("parsing field : {}", f.name());
            let res = match f.parse(&mut cp_data, &mut iso_msg) {
                Err(e) => Result::Err(e),
                Ok(r) => Result::Ok(r),
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