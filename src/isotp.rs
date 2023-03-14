use std::{
    thread,
    time::{self, Duration},
};

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

    fn read_from_can(&self) -> Option<IsoTPFrame> {
        if let Ok(frame) = self.cansocket.read() {
            return Some(frame.into());
        }
        return None;
    }

    fn send_consecutive_frames(&self, frame: &mut IsoTPFrame) {
        //Send First Frame
        self.send_on_can(FrameType::FirstFrame(frame, self));
        //Await Flow Control
        self.consecutive_send(frame);
    }

    fn consecutive_send(&self, data_frame: &mut IsoTPFrame) {
        let mut ct: bool;
        loop {
            let fc_frame: FlowControlFrame;
            if let Some(frame) = self.next_fc_frame() {
                fc_frame = frame.into();
            } else {
                panic!("shit frame")
            }

            match fc_frame.transfer_allowed {
                Some(FlowControlType::WAIT) => ct = false,
                Some(FlowControlType::ABORT) => return,
                Some(FlowControlType::CONTINUE) => ct = true,
                None => return,
            }

            if ct {
                let mut idx = 0u8;
                loop {
                    if idx == fc_frame.block_size && fc_frame.block_size != 0 {
                        break;
                    }
                    thread::sleep(self.calc_sleep_dur(fc_frame.seperation_time));
                    self.send_on_can(FrameType::ConsecutiveFrame(data_frame, self));
                    idx += 1;
                }
            }
        }
    }

    fn next_fc_frame(&self) -> Option<IsoTPFrame> {
        self.read_from_can()
    }

    fn calc_sleep_dur(&self, time: u8) -> Duration {
        match time {
            0..=127 => time::Duration::from_millis(time as u64),
            241 => time::Duration::from_micros(100),
            242 => time::Duration::from_micros(200),
            243 => time::Duration::from_micros(300),
            244 => time::Duration::from_micros(400),
            245 => time::Duration::from_micros(500),
            246 => time::Duration::from_micros(600),
            247 => time::Duration::from_micros(700),
            248 => time::Duration::from_micros(800),
            249 => time::Duration::from_micros(900),
            _ => time::Duration::from_micros(0),
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
        sf_data.push(self.size as u8);
        self.data
            .drain(0..FF_DATA_SIZE)
            .for_each(|e| sf_data.push(e));

        return sf_data;
    }

    fn create_cf(&mut self) -> Vec<u8> {
        let mut sf_data: Vec<u8> = Vec::new();
        sf_data.push(((FrameTypeValue::CONSECUTIVE as u8) << 4) + (self.idx));
        self.idx += 1;
        let mut drainsize = CF_DATA_SIZE;
        if self.data.len() <= CF_DATA_SIZE {
            drainsize = self.data.len();
        }

        self.data.drain(0..drainsize).for_each(|e| sf_data.push(e));

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

pub struct FlowControlFrame {
    frame_type: FrameTypeValue,
    transfer_allowed: Option<FlowControlType>,
    block_size: u8,
    seperation_time: u8,
}

#[derive(FromPrimitive)]
enum FlowControlType {
    CONTINUE = 1,
    WAIT = 2,
    ABORT = 3,
}

impl FlowControlFrame {
    pub fn new(transfer_allowed: u8, block_size: u8, seperation_time: u8) -> Self {
        Self {
            frame_type: FrameTypeValue::FLOW,
            transfer_allowed: num::FromPrimitive::from_u8(transfer_allowed),
            block_size,
            seperation_time,
        }
    }
}

impl From<CanFrame> for IsoTPFrame {
    fn from(value: CanFrame) -> Self {
        return IsoTPFrame::new(value.data.to_vec());
    }
}

impl From<IsoTPFrame> for FlowControlFrame {
    fn from(value: IsoTPFrame) -> Self {
        return FlowControlFrame::new(value.data[0].to_le_bytes()[0], value.data[1], value.data[2]);
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
        vec![
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 12, 123, 123, 1, 66, 88, 99, 213, 12, 12,
        ]
    }
}
