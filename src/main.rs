use std::io::{self};

use etherparse::{IpNumber, Ipv4HeaderSlice, TcpHeaderSlice};
use tun_tap::{Iface, Mode};

fn main() -> io::Result<()> {
    let iface = Iface::new("tun0", Mode::Tun).expect("Failed to create a TUN device");

    // Etherenet MTU is typically 1500 bytes + 4 for header
    // Flags [2 bytes]
    // Proto [2 bytes] => EtherType
    // Raw protocol(IP, IPv6, etc) frame {1500bytes}
    let mut buffer = vec![0u8; 1504];

    loop {
        let nbytes = iface.recv(&mut buffer).unwrap();
        if nbytes == 0 {
            break;
        }
        // Network protocols send numbers as bytes, not integers.
        // from_be_bytes gives those bytes semantic meaning.
        // let ethernet_flags = u16::from_be_bytes([buffer[0], buffer[1]]);
        let ethernet_proto = u16::from_be_bytes([buffer[2], buffer[3]]);
        if ethernet_proto == 0x800 {
            // Only for IPv4 protocol
            //0x800 => Ethertype in the Ethernet frame tells the reciever which protocol the ip_header belongs to
            match Ipv4HeaderSlice::from_slice(&buffer[4..nbytes]) {
                Err(value) => eprintln!("Err {:?}", value),
                Ok(ip_header) => {
                    let ip_proto = ip_header.protocol();

                    if ip_proto == IpNumber::from(6) {
                        // Only catch for TCP
                        match TcpHeaderSlice::from_slice(&buffer[4 + ip_header.slice().len()..]) {
                            Ok(tcp_payload) => {
                                println!("Got TCP ip_header: {:?}", tcp_payload);
                            }
                            Err(value) => eprintln!("Err {:?}", value),
                        }
                    } else {
                        continue;
                    }
                }
            }
        } else {
            continue;
        }
    }

    Ok(())
}
