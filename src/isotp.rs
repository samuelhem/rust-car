use crate::cansocket::{CanFrame, CanSocket};

pub struct IsoTPSocket {
    cansocket: CanSocket,
    destination_id: u32,
    source_id: u32,
}

impl IsoTPSocket {
    pub fn new(can_socket_name: &str, destination_id: u32, source_id: u32) -> Self {
        Self {
            cansocket: CanSocket::new(can_socket_name),
            destination_id,
            source_id,
        }
    }

    pub fn send<T: Sendable>(&self, frame: T) {
        let mut iso_tp_frame = IsoTPFrame::new(frame.convert());

        match iso_tp_frame.size {
            0..=7 => self.send_on_can(FrameType::SingleFrame(iso_tp_frame, self)),
            8..=4095 => self.send_consecutive_frames(&mut iso_tp_frame),
            _ => {}
        }
    }

    fn send_on_can(&self, frame: FrameType) {
        if let Ok(_) = self.cansocket.send(&frame.into()) {
            println!("Frame successfully sent");
        } else {
            println!("Error sending Frame")
        }
    }

    fn send_consecutive_frames(&self, frame: &mut IsoTPFrame) {
        self.send_on_can(FrameType::FirstFrame(frame, self));
        while frame.data.len() > 0 {
            self.send_on_can(FrameType::ConsecutiveFrame(frame, self));
        }
    }
}

pub struct IsoTPFrame {
    data: Vec<u8>,
    size: usize,
    idx: u8,
}

const FF_DATA_SIZE: usize = 5;
const CF_DATA_SIZE: usize = 6;

impl IsoTPFrame {
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            data: data.to_owned(),
            size: data.len(),
            idx: 0,
        }
    }

    fn convert_data_to_sf(&self) -> Vec<u8> {
        let mut sf_data: Vec<u8> = Vec::new();
        sf_data.push(((FrameTypeValue::SINGLE as u8) << 4) + self.size as u8);
        self.data.iter().for_each(|e| sf_data.push(*e));
        return sf_data;
    }

    fn create_ff(&mut self) -> Vec<u8> {
        let mut sf_data: Vec<u8> = Vec::new();
        sf_data.push(((FrameTypeValue::FIRST as u8) << 4) + (self.size >> 8) as u8);
        sf_data.extend((self.size as u16).to_be_bytes().iter());
        self.data
            .drain(0..FF_DATA_SIZE)
            .for_each(|e| sf_data.push(e));

        return sf_data;
    }

    fn create_cf(&mut self) -> Vec<u8> {
        let mut sf_data: Vec<u8> = Vec::new();
        sf_data.push(FrameTypeValue::CONSECUTIVE as u8);
        sf_data.push(self.idx);
        self.idx += 1;
        self.data
            .drain(0..CF_DATA_SIZE)
            .for_each(|e| sf_data.push(e));

        return sf_data;
    }
}

pub enum FrameType<'a> {
    SingleFrame(IsoTPFrame, &'a IsoTPSocket),
    FirstFrame(&'a mut IsoTPFrame, &'a IsoTPSocket),
    ConsecutiveFrame(&'a mut IsoTPFrame, &'a IsoTPSocket),
    FlowControlFrame(IsoTPFrame, &'a IsoTPSocket),
}

enum FrameTypeValue {
    SINGLE = 0,
    FIRST = 1,
    CONSECUTIVE = 2,
    FLOW = 3,
}

impl<'a> From<FrameType<'a>> for CanFrame {
    fn from(value: FrameType) -> Self {
        let data: Vec<u8>;
        let socket: &IsoTPSocket;
        match value {
            FrameType::SingleFrame(frame, s) => {
                data = frame.convert_data_to_sf();
                socket = s;
            }
            FrameType::FirstFrame(frame, s) => {
                data = frame.create_ff();
                socket = s;
            }
            FrameType::ConsecutiveFrame(frame, s) => {
                data = frame.create_cf();
                socket = s;
            }
            FrameType::FlowControlFrame(..) => todo!(),
        }
        return CanFrame::new(socket.destination_id, data.len() as u8).with_data(data);
    }
}

pub trait Sendable {
    fn convert(&self) -> Vec<u8>;
}

pub struct UDSFrame {
    service_id: u32,
}

impl UDSFrame {
    pub fn new(service_id: u32) -> Self {
        Self { service_id }
    }
}

impl Sendable for UDSFrame {
    fn convert(&self) -> Vec<u8> {
        vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 12, 123, 123]
    }
}
