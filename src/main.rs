use isotp::IsoTPSocket;
use rust_can_bindings::{isotp, uds::UDSFrame};

extern crate num;
#[macro_use]
extern crate num_derive;

fn main() {
    /*let socket = CanSocket::new("vcan0");

    let mut frame = CanFrame::new(0x143, 2);

    frame.data = "Hello World"
        .parse::<BinaryModel>()
        .unwrap()
        .convert_to_frame_data();
        */
    /*loop {
    frame.data[1] = frame.data[1] + 1;
    socket.send(&frame);
    sleep(std::time::Duration::from_secs(1));

    println!("{:?}", socket.read().unwrap());
    }*/

    let isotp_socket = IsoTPSocket::new("vcan0", 0x700, 0x600);
    let frame = UDSFrame::new(
        0x27,
        String::from("Halloooooo, ich bin ein UDS Frame über ISO-TP und CAN gesendet"),
    );

    isotp_socket.send(frame);
}
