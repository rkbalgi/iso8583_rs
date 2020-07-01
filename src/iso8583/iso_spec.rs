//! This module contains implementation of specification, its segments and associated operations
//!
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::io::Cursor;

use crate::iso8583::{bitmap, IsoError};
use crate::iso8583::field::{Field, ParseError};
use crate::iso8583::yaml_de::YMessageSegment;
use crate::iso8583::bitmap::Bitmap;
use crate::iso8583::config::Config;
use crate::crypto::pin::generate_pin_block;

// Reads the spec definitions from YAML file
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
     Ok(spec)=> specs.insert(String::from(spec.name()),spec),
     Err(e)=> panic!(e.msg)
    };

    specs
};
}

/// This struct is the definition of the specification - layout of fields etc..
pub struct Spec {
    pub(in crate::iso8583) name: String,
    #[allow(dead_code)]
    pub(in crate::iso8583) id: u32,
    pub(in crate::iso8583) messages: Vec<MessageSegment>,
    pub(in crate::iso8583) header_fields: Vec<Box<dyn Field>>,
}

/// This struct represents a segment in the Spec (a auth request, a response etc)
pub struct MessageSegment {
    pub(in crate::iso8583) name: String,
    #[allow(dead_code)]
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


/// Operations on MessageSegment
impl MessageSegment {
    /// Returns name of segment
    pub fn name(&self) -> &str {
        return self.name.as_str();
    }

    /// Returns a field given it's name if defined in the spec or a IsoError if the field is not found
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

    /// Returns a message segment given its name or a IsoError if such a segment is not present
    pub fn get_message(&self, name: &str) -> Result<&MessageSegment, IsoError> {
        for msg in &self.messages {
            if msg.name() == name {
                return Ok(msg);
            }
        }
        return Err(IsoError { msg: format!("{} message not found", name) });
    }

    /// Returns a message that corresponds to the given header value or an IsoError if such a selector
    /// doesn't exist
    pub fn get_message_from_header(&self, header_val: &str) -> Result<&MessageSegment, IsoError> {
        for msg in &self.messages {
            if msg.selector.contains(&header_val.to_string()) {
                return Ok(msg);
            }
        }
        return Err(IsoError { msg: format!("message not found for header - {}", header_val) });
    }

    /// Returns a segment by first parsing the header field and then matching the header value against
    /// the selector
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

/// This struct represents a parsed message for a given spec
pub struct IsoMsg {
    // The spec associated with this IsoMsg
    pub spec: &'static Spec,
    /// The segment that the IsoMsg represents
    pub msg: &'static MessageSegment,
    /// field data map - name to raw value
    pub fd_map: std::collections::HashMap<String, Vec<u8>>,
    /// the bitmap on the iso message
    pub bmp: bitmap::Bitmap,
}

/// Operations on IsoMsg
impl IsoMsg {
    pub fn spec(&self) -> &'static Spec {
        self.spec
    }

    /// Returns the value of a field by position in the bitmap
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

    /// Returns the value of a top level field like message_type
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

    /// sets a top-level field like message_type etc
    pub fn set(&mut self, name: &str, val: &str) -> Result<(), IsoError> {
        match self.msg.field_by_name(&name.to_string()) {
            Ok(f) => {
                self.fd_map.insert(f.name().clone(), f.to_raw(val));
                Ok(())
            }
            Err(e) => Err(e)
        }
    }

    /// Sets a field in the bitmap with the given value
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

    /// Echoes (sets the value with the identical field in req_msg) for given positions in the bitmap
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

    /// Assembles the messages into a Vec<u8> or a IsoError on failure
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

    /// Sets F52 based on provided clear pin, and format/key provided via cfg
    pub fn set_pin(&mut self, pin: &str, pan: &str, cfg: &Config) -> Result<(), IsoError> {

        if cfg.get_pin_fmt().is_none() || cfg.get_pin_key().is_none() {
            return Err(IsoError { msg: format!("missing pin_format or key in call to set_pin") });
        }

        match generate_pin_block(&cfg.get_pin_fmt().as_ref().unwrap(), pin, pan, cfg.get_pin_key().as_ref().unwrap().as_str()) {
            Ok(v) => {
                self.set_on(52, hex::encode(v).as_str())
            }
            Err(e) => {
                Err(IsoError { msg: e.msg })
            }
        }
    }
}

fn collect_children(f: &dyn Field, ordered_fields: &mut Vec<String>) {
    ordered_fields.push(f.name().clone());
    f.children().iter().for_each(|f| collect_children(*f, ordered_fields));
}

impl Display for IsoMsg {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        let mut res = "".to_string();
        let mut ordered_fields = vec![];
        self.msg.fields.iter().for_each(|f| collect_children(f.as_ref(), &mut ordered_fields));

        res = res + format!("\n{:20.40} : {:5}  : {} ", "-Field-", "-Position-", "-Field Value-").as_str();
        for f in ordered_fields {
            if self.fd_map.contains_key(f.as_str()) {
                let field = self.msg.field_by_name(&f).unwrap();
                let field_value = &self.fd_map.get(f.as_str()).unwrap();
                let mut pos_str: String = String::new();
                if field.position() > 0 {
                    pos_str = format!("{:03}", field.position());
                }

                //debug!("** formatting {}",field.name());
                res = res + format!("\n{:20.40} : {:^10}  : {} ", f, pos_str.as_str(), field.to_string(field_value)).as_str();
            }
        }
        f.write_str(&res).unwrap();
        Ok(())
    }
}

/// Returns a spec given its name
pub fn spec(_name: &str) -> &'static Spec {
    //TODO:: handle case of multiple specs, for now just return the first
    ALL_SPECS.iter().find_map(|(_k, v)| Some(v)).unwrap()
}

/// Returns a empty IsoMsg that can be used to create a message
pub fn new_msg(spec: &'static Spec, seg: &'static MessageSegment) -> IsoMsg {
    IsoMsg {
        spec,
        msg: seg,
        fd_map: HashMap::new(),
        bmp: Bitmap::new(0, 0, 0),
    }
}

impl Spec {
    /// Returns a IsoMsg after parsing data or an ParseError on failure
    pub fn parse(&'static self, data: &mut Vec<u8>) -> Result<IsoMsg, ParseError> {
        let msg = self.get_msg_segment(data);
        if msg.is_err() {
            return Err(ParseError { msg: msg.err().unwrap().msg });
        }

        let mut iso_msg = IsoMsg {
            spec: &self,
            msg: &msg.unwrap(),
            fd_map: HashMap::new(),
            bmp: Bitmap::new(0, 0, 0),
        };

        let mut cp_data = Cursor::new(data);

        for f in &iso_msg.msg.fields {
            debug!("parsing field : {}", f.name());
            let res = match f.parse(&mut cp_data, &mut iso_msg.fd_map) {
                Err(e) => Result::Err(e),
                Ok(_) => {
                    //if this is "THE" bitmap, then save it on isomsg
                    if f.name() == "bitmap" {
                        let bmp_data = iso_msg.fd_map.get(f.name()).unwrap();
                        iso_msg.bmp = Bitmap::from_vec(bmp_data);
                    }
                    Ok(())
                }
            };

            if res.is_err() {
                return Result::Err(res.err().unwrap());
            }
        }
        Ok(iso_msg)
    }
}