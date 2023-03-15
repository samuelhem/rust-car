use aligned_array::{Aligned, A8};
use nix::net::if_::if_nametoindex;
use std::mem::size_of;
use std::{
    io::{Error, ErrorKind},
    str::FromStr,
};

const SOCK_RAW: i32 = 3;
const CAN_RAW: i32 = 1;
const PF_CAN: i32 = 29;

pub struct CanSocket {
    socket: i32,
}

impl CanSocket {
    pub fn new(name: &str) -> Self {
        let s: i32;
        unsafe {
            s = socket(PF_CAN, SOCK_RAW, CAN_RAW);
        }
        if s == -1 {
            panic!("Error while opening socket");
        }

        let if_index = if_nametoindex(name).expect("No device with the name found");
        let sock_addr = SocketAddrCan::new(PF_CAN, if_index);

        let res: i32;
        unsafe {
            res = bind(
                s,
                &sock_addr as *const SocketAddrCan,
                size_of::<SocketAddrCan>(),
            );
        }
        if res == -1 {
            panic!("Error binding to socket");
        }
        return CanSocket { socket: s };
    }

    pub fn send(&self, frame: &CanFrame) -> Result<i32, std::io::Error> {
        let bytes: i32;
        unsafe {
            bytes = write(self.socket, frame as *const CanFrame, size_of::<CanFrame>());
        }
        if bytes == -1 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "Error sending data to bus",
            ));
        }
        Ok(bytes)
    }

    pub fn read(&self) -> std::io::Result<CanFrame> {
        let mut frame = CanFrame {
            can_id: 0,
            can_dlc: 0,
            data: Aligned([0; 8]),
        };

        let read_size: i32;
        unsafe {
            read_size = read(
                self.socket,
                &mut frame as *mut CanFrame,
                size_of::<CanFrame>(),
            );
        }
        if read_size as usize != size_of::<CanFrame>() {
            return Err(Error::last_os_error());
        }

        Ok(frame)
    }
}

#[repr(C)]
struct SocketAddrCan {
    can_family: i32,
    can_ifindex: u32,
}

impl SocketAddrCan {
    fn new(family: i32, index: u32) -> Self {
        return SocketAddrCan {
            can_family: family,
            can_ifindex: index,
        };
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct CanFrame {
    pub can_id: u32,
    pub can_dlc: u8,
    pub data: Aligned<A8, [u8; 8]>,
}

impl CanFrame {
    pub fn new(can_id: u32, can_dlc: u8) -> Self {
        return CanFrame {
            can_id,
            can_dlc,
            data: Aligned([0; 8]),
        };
    }

    pub fn with_data(mut self, data: Vec<u8>) -> Self {
        self.data = BinaryModel::convert_to_array(data);
        return self;
    }
}

#[derive(Debug)]
pub struct BinaryModel {
    value: String,
    bytes: Vec<u8>,
}

impl BinaryModel {
    fn new(value: String, bytes: Vec<u8>) -> Self {
        Self { value, bytes }
    }
}

#[derive(Debug)]
pub struct ParseIntoBinaryModelErr;

impl FromStr for BinaryModel {
    type Err = ParseIntoBinaryModelErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes: Vec<u8> = s.bytes().into_iter().collect();
        Ok(BinaryModel::new(String::from(s), bytes))
    }
}

impl BinaryModel {
    pub fn convert_to_frame_data(&self) -> Aligned<A8, [u8; 8]> {
        let ret: [u8; 8] = self.bytes[0..8].try_into().expect("incorrect length");
        Aligned(ret)
    }

    pub fn convert_to_array(data: Vec<u8>) -> Aligned<A8, [u8; 8]> {
        let mut ret = [0; 8];
        data.iter().enumerate().for_each(|(i, x)| ret[i] = *x);
        Aligned(ret)
    }
}

extern "C" {
    fn socket(domain: i32, t: i32, protocol: i32) -> i32;
}

extern "C" {
    fn bind(socket: i32, address: *const SocketAddrCan, address_len: usize) -> i32;
}

extern "C" {
    fn write(socket: i32, buf: *const CanFrame, count: usize) -> i32;
}

extern "C" {
    fn read(socket: i32, buf: *mut CanFrame, size: usize) -> i32;
}
