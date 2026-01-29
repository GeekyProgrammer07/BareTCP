use std::{
    collections::{HashMap, hash_map::Entry},
    error::Error,
    net::Ipv4Addr,
};

use etherparse::{IpNumber, Ipv4HeaderSlice, TcpHeaderSlice};
use tun_tap::{Iface, Mode};

use crate::tcp::connection::Connection;

mod tcp;

#[derive(Eq, Hash, PartialEq)]
struct Quad {
    // Quad: (SrcIp, SrcPort, DesIp, DesPort)
    src: (Ipv4Addr, u16),
    dst: (Ipv4Addr, u16),
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut iface = Iface::new("tun0", Mode::Tun).expect("Failed to create a TUN device"); // Etherenet MTU is typically 1500 bytes + 4 for header // Flags [2 bytes] // Proto [2 bytes] => EtherType // Raw protocol(IP, IPv6, etc) frame {1500bytes}
    let mut buffer = vec![0u8; 1504];
    let mut connections: HashMap<Quad, Connection> = Default::default();

    loop {
        let nbytes = iface.recv(&mut buffer).unwrap();
        if nbytes == 0 {
            break;
        }

        // Network protocols send numbers as bytes, not integers.
        // from_be_bytes gives those bytes semantic meaning.
        // let ethernet_flags = u16::from_be_bytes([buffer[0], buffer[1]]);
        let ethernet_proto = u16::from_be_bytes([buffer[2], buffer[3]]);
        if ethernet_proto != 0x800 {
            continue;
        }

        // Only for IPv4 protocol
        //0x800 => Ethertype in the Ethernet frame tells the reciever which protocol the ip_header belongs to
        let ip_header = match Ipv4HeaderSlice::from_slice(&buffer[4..nbytes]) {
            Ok(h) => h,
            Err(value) => {
                eprintln!("Err {:?}", value);
                continue;
            }
        };

        let ip_proto = ip_header.protocol();
        if ip_proto != IpNumber::from(6) {
            continue;
        }

        // Only catch for TCP
        let tcp_header = match TcpHeaderSlice::from_slice(&buffer[4 + ip_header.slice().len()..]) {
            Ok(h) => h,
            Err(value) => {
                eprintln!("Err {:?}", value);
                continue;
            }
        };

        // let header_data =
        //     4 + ip_header.slice().len() + tcp_header.slice().len();

        match connections.entry(Quad {
            src: (ip_header.source_addr(), tcp_header.source_port()),
            dst: (ip_header.destination_addr(), tcp_header.destination_port()),
        }) {
            Entry::Occupied(_c) => {
                // Update the states
                println!("Over here we update the states");
            }
            Entry::Vacant(e) => {
                // There is no connection I am creating one
                if let Some(conn) = Connection::accept(&mut iface, &ip_header, &tcp_header)? {
                    e.insert(conn);
                }
            }
        }
    }

    Ok(())
}
