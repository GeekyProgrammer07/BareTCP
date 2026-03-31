use etherparse::{Ipv4HeaderSlice, TcpHeaderSlice};
use tun_tap::Iface;

use crate::{
    tcp::{output::send_syn_ack, recv::RecvSequenceSpace, send::SendSequenceSpace, state::State},
    util::{
        isn::generate_isn,
        seq::{seq_le, seq_lt},
    },
};

// Connection {
//     identity (4-tuple)
//     state (FSM)
//     send_seq_space
//     recv_seq_space
//     send_buffer
//     recv_buffer
// }
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
        if !tcp_header.syn() {
            // Returns if its not a SYN packet
            return Ok(None);
        }

        let iss = generate_isn();

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

        send_syn_ack(nic, ip_header, tcp_header, iss);

        Ok(Some(c))
    }

    pub fn on_packet(
        &mut self,
        tcp_header: &TcpHeaderSlice,
        data: &[u8],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let seg_ack = tcp_header.acknowledgment_number();
        // First check that the sequence numbers are valid
        // Acceptable ACK => SND.UNA < SEG.ACK =< SND.NXT
        // Including wraparound
        if !(seq_lt(self.send.una, seg_ack) && seq_le(seg_ack, self.send.nxt)) {
            // Not a valid squence number
            return Ok(());
        }

        let seg_seq = tcp_header.sequence_number();
        // Validating whether an incoming segment (SEG) is acceptable
        // based on:
        //   - Segment length (SEG.LEN)
        //   - Receive window size (RCV.WND)
        //
        // A segment is considered valid (acceptable) if any part of it falls inside the receive window.
        //
        // Special cases exist for zero-length segments and zero window size:
        //
        // ┌───────────────┬───────────────┬────────────────────────────────────────────┐
        // │ SEG.LEN       │ RCV.WND       │ Acceptability condition                    │
        // ├───────────────┼───────────────┼────────────────────────────────────────────┤
        // │ 0             │ 0             │ SEG.SEQ == RCV.NXT                         │
        // │ 0             │ >0            │ RCV.NXT ≤ SEG.SEQ < RCV.NXT+RCV.WND        │
        // │ >0            │ 0             │ NOT acceptable                             │
        // │ >0            │ >0            │ Segment overlaps window:                   │
        // │               │               │   start ∈ window OR end ∈ window           │
        // └───────────────┴───────────────┴────────────────────────────────────────────┘
        // start ∈ window OR end ∈ window => RCV.NXT =< SEG.SEQ < RCV.NXT+RCV.WND or RCV.NXT =< SEG.SEQ+SEG.LEN-1 < RCV.NXT+RCV.WND
        //
        // NOTE:
        // - "Zero-length segment" here means:
        //     no payload AND no SYN AND no FIN
        // - SYN and FIN each consume 1 byte in sequence space, even without payload

        if data.len() == 0 && !tcp_header.syn() && !tcp_header.fin() {
            // Case 1 & 2: SEG.LEN == 0

            if self.receive.wnd == 0 {
                // Case 1: LEN=0, WND=0
                // Only acceptable if it exactly matches the next expected sequence
                if seg_seq != self.receive.nxt {
                    return Ok(());
                }
            } else if !(seq_le(self.receive.nxt, seg_seq)
                && seq_lt(seg_seq, self.receive.nxt + u32::from(self.receive.wnd)))
            {
                // Case 2: LEN=0, WND>0
                // Accept only if SEG.SEQ lies within receive window
                return Ok(());
            }
        } else {
            // Case 3 & 4: SEG.LEN > 0 (data or SYN/FIN present)

            if self.receive.wnd == 0 {
                // Case 3: LEN>0, WND=0
                // No data is acceptable when window is zero
                return Ok(());
            } else if !(
                // Case 4: LEN>0, WND>0
                // Accept if ANY part of segment overlaps the window

                // Check 1: Segment end is inside window
                (seq_le(self.receive.nxt, seg_seq + data.len() as u32 - 1)
            && seq_lt(
                seg_seq + data.len() as u32 - 1,
                self.receive.nxt + u32::from(self.receive.wnd),
            ))

        // Check 2: Segment start is inside window
        || (seq_le(self.receive.nxt, seg_seq)
            && seq_lt(seg_seq, self.receive.nxt + u32::from(self.receive.wnd)))
            ) {
                // If neither start nor end lies in the window then reject
                return Ok(());
            }
        }

        match self.state {
            State::SynRcvd => {
                // Third handshake of 3-way handshake
                // expect to recieve an ack for our syn ack
                if !tcp_header.ack() {
                    return Ok(());
                }

                if seg_ack == self.send.iss + 1 {
                    // Changing the state
                    self.state = State::Estab;
                    self.send.una = seg_ack;
                    println!("Changed State to Estab");
                    println!("{:?}", self.state);
                }
            }
            _ => {}
        }
        Ok(())
    }
}
