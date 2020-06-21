pub mod bitmap;
pub mod field;
pub mod iso_spec;
pub mod server;
mod test;
pub mod msg_processor;
mod yaml_de;

#[derive(Debug)]
pub struct IsoError{
    pub msg: String,
}
