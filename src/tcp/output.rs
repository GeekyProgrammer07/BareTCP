use etherparse::{IpNumber, Ipv4Header, Ipv4HeaderSlice, TcpHeader, TcpHeaderSlice};
use tun_tap::Iface;

pub fn send_tcp_packet(
    nic: &mut Iface,
    ip_header: &Ipv4HeaderSlice,
    tcp_header: &mut TcpHeader,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer = [0u8; 1504]; // Not 1504 cause of the exclusion of ethernet stuffs

    // TUN packet info header
    buffer[0] = 0x00;
    buffer[1] = 0x00;
    buffer[2] = 0x08;
    buffer[3] = 0x00;

    tcp_header.checksum = TcpHeader::calc_checksum_ipv4(tcp_header, &ip_header.to_header(), &[])?;

    let ip_packet = Ipv4Header::new(
        tcp_header.header_len() as u16 + 0, // SYN or SYN-ACK doesn't have any data so 0(not needed)
        64,                                 // TTL => is the hop count
        IpNumber::TCP,
        ip_header.destination(),
        ip_header.source(),
    )?;

    let unwritten = {
        let mut unwritten = &mut buffer[4..];
        ip_packet.write(&mut unwritten)?;
        tcp_header.write(&mut unwritten)?;

        unwritten.len()
    };

    nic.send(&buffer[..buffer.len() - unwritten])?;
    Ok(())
}

pub fn send_syn_ack(
    nic: &mut Iface,
    ip_header: &Ipv4HeaderSlice,
    tcp_header: &TcpHeaderSlice,
    iss: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    // Send a SYN-ACK packet (2 way handshake)
    let mut syn_ack = TcpHeader::new(
        tcp_header.destination_port(),
        tcp_header.source_port(),
        iss, // Random as we are sending a just a syn-ack
        10,  // Keeping 10 for initial phases TODO: Will change later with specs
    );
    syn_ack.acknowledgment_number = tcp_header.sequence_number() + 1;
    syn_ack.sequence_number = iss;
    syn_ack.ack = true;
    syn_ack.syn = true;

    send_tcp_packet(nic, ip_header, &mut syn_ack)
}

// pub fn send_rst(
//     nic: &mut Iface,
//     ip_header: &Ipv4HeaderSlice,
//     tcp_header: &TcpHeaderSlice,
//     iss: u32,
// ) -> Result<(), Box<dyn std::error::Error>> {
// }
