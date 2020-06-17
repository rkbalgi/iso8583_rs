pub mod bitmap;
pub mod field;
pub mod iso_spec;
pub mod server;
mod test;
mod msg_processor;

#[derive(Debug)]
pub struct IsoError{
    pub msg: String,
}
