//#[macro_use]
//extern crate lazy_static;

use crate::iso8583::field::{FixedField, Field};

use crate::iso8583::field::Encoding;
use std::error::Error;
use std::string::ParseError;
use std::collections::HashMap;

pub struct Spec {
    name: String,
    fields: Vec<Box<dyn Field>>,
}



lazy_static! {
static ref all_specs: std::collections::HashMap<String,Spec> ={

    let mut specs=HashMap::new();

    specs.insert("SampleSpec".to_string(),Spec {
        name: "SampleSpec".to_string(),
        fields: vec![
            Box::new(FixedField { name: "f1".to_string(), len: 1, encoding: Encoding::ASCII }),
            Box::new(FixedField { name: "f2".to_string(), len: 2, encoding: Encoding::ASCII }),
            Box::new(FixedField { name: "f3".to_string(), len: 3, encoding: Encoding::ASCII }),
            Box::new(FixedField { name: "f4".to_string(), len: 4, encoding: Encoding::ASCII }),
            Box::new(FixedField { name: "f5".to_string(), len: 5, encoding: Encoding::ASCII }),
            Box::new(FixedField { name: "f6".to_string(), len: 6, encoding: Encoding::ASCII }),
        ],
    });
    specs
};
}


pub fn Spec(name: String) -> &'static Spec {
    return all_specs.get(&name).unwrap();
}

impl Spec {
    pub fn parse(&self, data: Vec<u8>) -> Result<u32, ParseError> {
        let mut cp_data = data.clone();

        for f in &self.fields {
            println!("parsing .. {}", f.name());
            match f.parse(&mut cp_data) {
                Err(e) => Result::Err(e),
                Ok(r) => Result::Ok(r),
            };
        }
        Ok(0)
    }
}