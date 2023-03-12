use std::thread::sleep;

use cansocket::{BinaryModel, CanFrame, CanSocket};
use isotp::{IsoTPFrame, IsoTPSocket, UDSFrame};
mod cansocket;
mod isotp;
fn main() {
    let socket = CanSocket::new("vcan0");

    let mut frame = CanFrame::new(0x143, 2);

    frame.data = "Hello World"
        .parse::<BinaryModel>()
        .unwrap()
        .convert_to_frame_data();

    /*loop {
    frame.data[1] = frame.data[1] + 1;
    socket.send(&frame);
    sleep(std::time::Duration::from_secs(1));

    println!("{:?}", socket.read().unwrap());
    }*/

    let isotp_socket = IsoTPSocket::new("vcan0", 0x700, 0x600);
    let frame = UDSFrame::new(0x12);

    isotp_socket.send(frame);
}
