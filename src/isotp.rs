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
        let iso_tp_frame = IsoTPFrame::new(frame.convert());

        match iso_tp_frame.size {
            0..=7 => self.send_on_can(FrameType::SingleFrame(iso_tp_frame, self)),
            8..=4095 => self.send_consecutive_frames(iso_tp_frame),
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

    fn send_consecutive_frames(&self, frame: IsoTPFrame) {}
}

pub struct IsoTPFrame {
    data: Vec<u8>,
    size: usize,
}

impl IsoTPFrame {
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            data: data.to_owned(),
            size: data.len(),
        }
    }
}

pub enum FrameType<'a> {
    SingleFrame(IsoTPFrame, &'a IsoTPSocket),
    FirstFrame(IsoTPFrame),
    ConsecutiveFrame(IsoTPFrame),
    FlowControlFrame(IsoTPFrame),
}

impl<'a> From<FrameType<'a>> for CanFrame {
    fn from(value: FrameType) -> Self {
        match value {
            FrameType::SingleFrame(frame, socket) => {
                return CanFrame::new(socket.destination_id, frame.size as u8)
                    .with_data(frame.data);
            }
            FrameType::FirstFrame(_) => todo!(),
            FrameType::ConsecutiveFrame(_) => todo!(),
            FrameType::FlowControlFrame(_) => todo!(),
        }
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
        vec![1, 2, 3, 4]
    }
}
