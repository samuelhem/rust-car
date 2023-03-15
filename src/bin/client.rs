use rust_can_bindings::{isotp::IsoTPSocket, uds::UDSFrame};

fn main() {
    let isotp_socket = IsoTPSocket::new("vcan0", 0x700, 0x600);

    let frame = isotp_socket.receive::<UDSFrame>();
    println!("{:?}", frame);
}
