pub mod mac;
pub mod pin;

extern crate rand;
extern crate des;

use des::TdesEde2;
use generic_array::GenericArray;
use des::block_cipher::NewBlockCipher;
use des::block_cipher::BlockCipher;

pub(crate) fn des_ede2_encrypt(data: &Vec<u8>, key: &Vec<u8>) -> Vec<u8> {
    let block_cipher = des::TdesEde2::new(GenericArray::from_slice(key.as_slice()));

    let mut cp_data = data.clone();
    block_cipher.encrypt_block(GenericArray::from_mut_slice(&mut cp_data));
    cp_data
}

pub(crate) fn des_ede2_decrypt(data: &Vec<u8>, key: &Vec<u8>) -> Vec<u8> {
    let block_cipher = des::TdesEde2::new(GenericArray::from_slice(key.as_slice()));

    let mut cp_data = data.clone();
    block_cipher.decrypt_block(GenericArray::from_mut_slice(&mut cp_data));
    cp_data
}