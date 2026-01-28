use etherparse::{IpNumber, Ipv4Header, Ipv4HeaderSlice, TcpHeader, TcpHeaderSlice};
use tun_tap::Iface;

pub enum State {
    Closed,
    Listen,
    SynRcvd,
    Estab,
}

impl Default for State {
    fn default() -> Self {
        State::Listen
    }
}

impl State {
    pub fn on_packet<'a>(
        &mut self,
        nic: &mut Iface,
        ip_header: &Ipv4HeaderSlice<'a>,
        tcp_header: &TcpHeaderSlice<'a>,
        data: &'a [u8],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut buffer = [0u8; 1500]; // Not 1504 cause of the exclusion of ethernet stuffs
        match self {
            Self::Closed => {
                return Ok(());
            }

            Self::Listen => {
                if !tcp_header.syn() {
                    // Returns if its not a SYN packet
                    println!("Not a SYN Packet");
                    return Ok(());
                }
                // Send a SYN-ACK packet (2 way handshake)
                println!("Its a SYN Packet");
                let mut syn_ack = TcpHeader::new(
                    tcp_header.destination_port(),
                    tcp_header.source_port(),
                    unimplemented!(),
                    unimplemented!(),
                );
                syn_ack.ack = true;
                syn_ack.syn = true;
                println!("syn_ack built");
                let mut ip_packet = Ipv4Header::new(
                    syn_ack.header_len() as u16 + 0, // SYN or SYN-ACK doesn't have any data so 0(not needed)
                    64,                              // TTL => is the hop count
                    IpNumber::TCP,
                    ip_header.destination(),
                    ip_header.source(),
                )?;
                println!("IP built");

                let written = {
                    let mut unwritten = &mut buffer[..];
                    ip_packet.write(&mut unwritten)?;
                    syn_ack.write(&mut unwritten)?;

                    buffer.len() - unwritten.len()
                };
                println!("IP Written to buffer");

                nic.send(&buffer[..written])?;
            }

            _ => {
                return Ok(());
            }
        }
    }
}
