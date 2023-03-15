use std::str;

use crate::isotp::{IsoTPFrame, Receivable, Sendable};

#[derive(Debug)]
pub struct UDSFrame {
    pub service_id: u16,
    pub service_data: String,
}

impl UDSFrame {
    pub fn new(service_id: u16, service_data: String) -> Self {
        Self {
            service_id,
            service_data,
        }
    }
}

impl Sendable for UDSFrame {
    fn convert(&self) -> Vec<u8> {
        let mut vec = (self.service_id).to_be_bytes().to_vec();
        vec.append(&mut self.service_data.as_bytes().to_vec());
        return vec;
    }
}

impl Receivable for UDSFrame {
    fn convert(f: IsoTPFrame) -> Self {
        UDSFrame {
            service_id: f.data[0..=1].iter().sum::<u8>() as u16,
            service_data: String::from(str::from_utf8(&f.data[2..f.size]).unwrap()),
        }
    }
}
