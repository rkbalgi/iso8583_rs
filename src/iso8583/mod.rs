pub mod client;
pub mod bitmap;
pub mod field;
pub mod iso_spec;
pub mod server;
mod test;
mod yaml_de;
pub mod mli;

/// IsoError represents a generic error throughout this and dependent sub-modules
#[derive(Debug)]
pub struct IsoError{
    pub msg: String,
}
