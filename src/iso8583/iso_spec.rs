//#[macro_use]
//extern crate lazy_static;

use crate::iso8583::field::{FixedField, VarField, BmpField, Field, Encoding, ParseError};
use crate::iso8583::bitmap;


use std::collections::HashMap;


pub struct Spec {
    name: String,
    fields: Vec<Box<dyn Field>>,
}

pub struct IsoMsg {
    pub spec: &'static Spec,
    pub fd_map: std::collections::HashMap<String, Vec<u8>>,
    pub bmp: bitmap::Bitmap,
}

impl IsoMsg {
    pub fn Spec(&self) -> &'static Spec {
        self.spec
    }
}



lazy_static! {
static ref all_specs: std::collections::HashMap<String,Spec> ={

    let mut specs=HashMap::new();

    let fields: Vec<Box<dyn Field>> = vec![
            Box::new(FixedField { name: "message_type".to_string(), len: 4, encoding: Encoding::ASCII ,position: 0}),
            Box::new(BmpField { name: "bitmap".to_string(), encoding: Encoding::ASCII ,
                 children: vec![Box::new(FixedField { name: "pan".to_string(), len: 12, encoding: Encoding::ASCII, position:2 }),
                                //Box::new(VarField { name: "pan".to_string(), len: 2, encoding: Encoding::ASCII, len_encoding: Encoding::ASCII, position:2 }),
                                Box::new(FixedField { name: "proc_code".to_string(), len: 6, encoding: Encoding::ASCII, position:3 }),
                                Box::new(FixedField { name: "stan".to_string(), len: 6, encoding: Encoding::ASCII, position:11 }),
                                Box::new(FixedField { name: "expiration_date".to_string(), len: 4, encoding: Encoding::ASCII, position: 14 }),
                               ]}),

        ];



    specs.insert("SampleSpec".to_string(),Spec {
        name: "SampleSpec".to_string(),
        fields: fields,
    });

    specs
};
}


pub fn Spec(name: &str) -> &'static Spec {
    return all_specs.get(name).unwrap();
}

impl Spec {
    pub fn fields(&self) -> &Vec<Box<dyn Field>> {
        &self.fields
    }
    pub fn parse(&'static self, data: Vec<u8>) -> Result<IsoMsg, ParseError> {
        let mut cp_data = data.clone();
        let mut iso_msg = IsoMsg { spec: &self, fd_map: HashMap::new(), bmp: bitmap::new_bmp(0, 0, 0) };

        for f in self.fields() {
            println!("parsing field : {}", f.name());
            match f.parse(&mut cp_data, &mut iso_msg) {
                Err(e) => Result::Err(e),
                Ok(r) => Result::Ok(r),
            };
        }

        if cp_data.len() > 0 {
            println!("residual data : {}", hex::encode(cp_data.iter()))
        }

        Ok(iso_msg)
    }
}