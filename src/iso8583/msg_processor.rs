use crate::iso8583::server::IsoServer;
use std::collections::HashMap;
use crate::iso8583::{bitmap, IsoError};
use crate::iso8583::iso_spec::IsoMsg;


// MsgProcessor is used by the IsoServer to handle incoming requests
pub trait MsgProcessor: Send + Sync {
    fn process(&self, iso_server: &IsoServer, msg: &mut Vec<u8>) -> Result<(Vec<u8>, IsoMsg), IsoError>;
}



