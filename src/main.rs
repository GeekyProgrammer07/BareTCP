use std::io::{self};

use etherparse::SlicedPacket;
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
        let flags = u16::from_be_bytes([buffer[0], buffer[1]]);
        let proto = u16::from_be_bytes([buffer[2], buffer[3]]);
        // println!("{:x}",proto);
        if proto == 0x800 {
            println!("inside");
            match SlicedPacket::from_ethernet(&buffer[4..nbytes]) {
                Err(value) => eprintln!("Err {:?}", value),
                Ok(value) => {
                    // println!(
                    //     "read {} bytes, flags: {}, proto: {:x}, BytesStream: {:02x?}",
                    //     nbytes,
                    //     flags,
                    //     proto,
                    //     &buffer[4..nbytes]
                    // );
                    println!(
                        "read {} bytes, flags: {}, proto: {:x}",
                        nbytes, flags, proto,
                    );
                    println!("link: {:?}", value.link);
                    println!("link_exts: {:?}", value.link_exts); // contains vlan & macsec
                    println!("net: {:?}", value.net); // contains ip & arp
                    println!("transport: {:?}", value.transport);
                }
            };
        } else {
            continue;
        }
    }

    Ok(())
}
