use crate::iso8583::field::{FixedField, Field};
use crate::iso8583::field::Encoding;
use std::error::Error;

struct Spec {
    name: String,
    fields: Vec<Field>,
}

static sampleSpec: Spec = Spec {
    name: "SampleSpec".to_string(),
    fields: vec![
        FixedField { name: "f1".to_string(), len: 1, encoding: Encoding::ASCII },
        FixedField { name: "f2".to_string(), len: 2, encoding: Encoding::ASCII },
        FixedField { name: "f3".to_string(), len: 3, encoding: Encoding::ASCII },
        FixedField { name: "f4".to_string(), len: 4, encoding: Encoding::ASCII },
        FixedField { name: "f5".to_string(), len: 5, encoding: Encoding::ASCII },
        FixedField { name: "f6".to_string(), len: 6, encoding: Encoding::ASCII },
    ],
};

pub fn Spec(name: String) -> &'static Spec {
    return &sampleSpec;
}

impl Spec {
    fn parse(&self, data: Vec<u8>) -> Result<u32, dyn Error> {
        for f in self.fields {
            //f.parse(data)?
            match f.parse(&data) {
                Err(E) => Err(E),
                Ok(R) => Ok(R),
            }
        }
        Ok(0)
    }
}