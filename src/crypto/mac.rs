//! This module provides implementation of MAC algorithms

//https://en.wikipedia.org/wiki/ISO/IEC_9797-1#Complete_specification_of_the_MAC_calculation

/// This enum defines various supported algorithms
pub enum MacAlgo {
    CBC_MAC,
    //ISO9797 - algo 1
    RETAIL_MAC, // ISO9797 - algo 3
}

/// This enum defines all supported padding types
pub enum PaddingType {
    /// Adding 0 bits
    Type1,
    /// Adding a single 1 bit followed by 0 bits
    Type2,
}

pub struct MacError {
    pub msg: String
}

pub fn generate_mac(algo: MacAlgo, padding_type: PaddingType, data: &Vec<u8>) -> Result<Vec<u8>, MacError> {
    let mut new_data = apply_padding(padding_type, data);
    Ok(new_data)
}

fn apply_padding(padding_type: PaddingType, data: &Vec<u8>) -> Vec<u8> {
    let mut new_data = data.clone();
    match padding_type {
        PaddingType::Type1 => {}
        PaddingType::Type2 => {
            new_data.push(0x80);
        }
    };

    while new_data.len() < 8 {
        new_data.push(0x00);
    }

    while new_data.len() % 8 != 0 {
        new_data.push(0x00);
    }

    new_data
}


#[cfg(test)]
mod tests {
    use crate::crypto::mac::{apply_padding, PaddingType};

    #[test]
    fn test_padding1_1() {
        let data = hex::decode("0102030405").unwrap();
        assert_eq!(hex::encode(apply_padding(PaddingType::Type1, &data)), "0102030405000000");
    }

    #[test]
    fn test_padding1_nopadding_reqd() {
        let data = hex::decode("0102030405060708").unwrap();
        assert_eq!(hex::encode(apply_padding(PaddingType::Type1, &data)), "0102030405060708");
    }

    #[test]
    fn test_padding1_2() {
        let data = hex::decode("0102030405060708090a").unwrap();
        assert_eq!(hex::encode(apply_padding(PaddingType::Type1, &data)), "0102030405060708090a000000000000");
    }


    #[test]
    fn test_padding2_1() {
        let data = hex::decode("0102030405").unwrap();
        assert_eq!(hex::encode(apply_padding(PaddingType::Type2, &data)), "0102030405800000");
    }

    #[test]
    fn test_padding2_exact() {
        let data = hex::decode("0102030405060708").unwrap();
        assert_eq!(hex::encode(apply_padding(PaddingType::Type2, &data)), "01020304050607088000000000000000");
    }

    #[test]
    fn test_padding2_2() {
        let data = hex::decode("0102030405060708090a").unwrap();
        assert_eq!(hex::encode(apply_padding(PaddingType::Type2, &data)), "0102030405060708090a800000000000");
    }
}