//! This module contains implementation of spec deserialization logic from a YAML file
use std::convert::TryInto;
use std::io::Read;

use serde::{Deserialize, Serialize};
use crate::iso8583::bitmap::BmpField;
use crate::iso8583::field::{Encoding, Field, FixedField, VarField};
use crate::iso8583::iso_spec::{MessageSegment, Spec};
use crate::iso8583::IsoError;

#[derive(Serialize, Deserialize)]
pub struct YField {
    pub name: String,
    pub id: u32,
    pub len: u32,
    #[serde(alias = "type")]
    pub field_type: String,
    pub len_encoding: Option<Encoding>,
    pub data_encoding: Encoding,
    pub position: Option<u32>,
    pub children: Option<Vec<YField>>,
}

impl Into<Box<dyn Field>> for &YField {
    fn into(self) -> Box<dyn Field> {
        match self.field_type.as_str() {
            "Fixed" => {
                Box::new(FixedField {
                    name: self.name.clone(),
                    id: self.id,
                    len: self.len,
                    encoding: self.data_encoding.clone(),
                    position: self.position.unwrap_or(0),
                })
            }
            "Variable" => {
                Box::new(VarField {
                    name: self.name.clone(),
                    id: self.id,
                    len: self.len,
                    len_encoding: self.len_encoding.unwrap(),
                    encoding: self.data_encoding.clone(),
                    position: self.position.unwrap_or(0),
                })
            }
            "Bitmapped" => {
                let mut children: Vec<Box<dyn Field>> = Vec::<Box<dyn Field>>::new();
                if self.children.is_some() {
                    let ychildren = &self.children.as_ref().unwrap();
                    &ychildren.iter().for_each(|f| {
                        children.push(Into::<Box<dyn Field>>::into(f));
                    });
                }

                Box::new(BmpField {
                    name: self.name.clone(),
                    id: self.id,
                    encoding: self.data_encoding.clone(),
                    children,
                })
            }
            _ => {
                panic!("Unsupported field type - {}", self.field_type.as_str());
            }
        }
    }
}


// Spec is the definition of the spec - layout of fields etc..
#[derive(Serialize, Deserialize)]
pub struct YSpec {
    pub(crate) name: String,
    pub(crate) id: u32,
    pub(crate) messages: Vec<YMessageSegment>,
    pub(crate) header_fields: Vec<YField>,
}


//impl From<&YSpec> for Spec {
impl Into<Spec> for YSpec {
    fn into(self) -> Spec {
        let mut header_fields: Vec<Box<dyn Field>> = vec![];

        self.header_fields.iter().for_each(|f| {
            header_fields.push(Into::<Box<dyn Field>>::into(f));
        });


        let mut messages: Vec<MessageSegment> = vec![];
        self.messages.iter().for_each(|m| {
            messages.push(MessageSegment::from(m));
        });


        Spec {
            name: self.name.clone(),
            id: self.id,
            messages,
            header_fields,
        }
    }
}


#[derive(Serialize, Deserialize)]
pub struct YMessageSegment {
    pub(crate) name: String,
    pub(crate) id: u32,
    pub(crate) selector: Vec<String>,
    pub(crate) fields: Vec<YField>,
}


pub fn read_spec(spec_file: &str) -> Result<Spec, IsoError> {
    match std::fs::File::open(spec_file) {
        Ok(f) => {
            let mut yaml_str = String::new();
            (&f).read_to_string(&mut yaml_str);

            match serde_yaml::from_str::<YSpec>(&yaml_str) {
                Ok(y_spec) => {
                    Ok(y_spec.into())
                }
                Err(e) => Err(IsoError { msg: e.to_string() })
            }
        }
        Err(e) => {
            Err(IsoError { msg: e.to_string() })
        }
    }
}


#[test]
fn test_deserialize_yaml_spec() {
    match read_spec("src\\iso8583\\sample_spec.yaml") {
        Ok(spec) => {
            assert_eq!(2, (&spec.messages).len());
        }
        Err(e) => assert!(false, e)
    }
}