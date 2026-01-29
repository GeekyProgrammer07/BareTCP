use etherparse::{IpNumber, Ipv4Header, Ipv4HeaderSlice, TcpHeader, TcpHeaderSlice};
use tun_tap::Iface;

use crate::tcp::{recv::RecvSequenceSpace, send::SendSequenceSpace, state::State};

pub struct Connection {
    state: State,
    send: SendSequenceSpace,
    receive: RecvSequenceSpace,
}

impl Connection {
    // Implement Later
    // pub fn passive_open() -> Self {
    //     Self {
    //         state: State::Listen,
    //         send: SendSequenceSpace::default(),
    //         receive: RecvSequenceSpace::default(),
    //     }
    // }

    // A SYN was received and we’re creating a connection
    pub fn accept<'a>(
        nic: &mut Iface,
        ip_header: &Ipv4HeaderSlice<'a>,
        tcp_header: &TcpHeaderSlice<'a>,
    ) -> Result<Option<Self>, Box<dyn std::error::Error>> {
        let mut buffer = [0u8; 1500]; // Not 1504 cause of the exclusion of ethernet stuffs

        if !tcp_header.syn() {
            // Returns if its not a SYN packet
            return Ok(None);
        }

        let iss = 0; // TODO: Change to a random value

        let c = Connection {
            state: State::SynRcvd,
            // Keep track of the sender info(Supose we are the server)
            send: SendSequenceSpace {
                iss,
                una: iss,
                nxt: iss + 1,
                wnd: 10, // TODO: Change this later
                up: None,
                wl1: 0,
                wl2: 0,
            },
            // Storing the info to be sent by the Reciever
            receive: RecvSequenceSpace {
                nxt: tcp_header.sequence_number() + 1,
                wnd: tcp_header.window_size(),
                up: tcp_header.urgent_pointer(),
                irs: tcp_header.sequence_number(),
            },
        };

        // Send a SYN-ACK packet (2 way handshake)
        let mut syn_ack = TcpHeader::new(
            tcp_header.destination_port(),
            tcp_header.source_port(),
            iss, // Random as we are sending a just a syn-ack TODO: Set it to some random val
            10,  // Keeping 10 for initial phases TODO: Will change later with specs
        );
        syn_ack.acknowledgment_number = tcp_header.sequence_number() + 1;
        syn_ack.sequence_number = iss;
        syn_ack.ack = true;
        syn_ack.syn = true;

        let ip_packet = Ipv4Header::new(
            syn_ack.header_len() as u16 + 0, // SYN or SYN-ACK doesn't have any data so 0(not needed)
            64,                              // TTL => is the hop count
            IpNumber::TCP,
            ip_header.destination(),
            ip_header.source(),
        )?;

        let unwritten = {
            let mut unwritten = &mut buffer[..];
            ip_packet.write(&mut unwritten)?;
            syn_ack.write(&mut unwritten)?;

            unwritten.len()
        };

        nic.send(&buffer[..buffer.len() - unwritten])?;
        Ok(Some(c))
    }
}
