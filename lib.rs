use std::collections::BTreeMap;
use std::io::prelude::*;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::{collections::HashMap, net::Ipv4Addr};
use std::{io, thread};

use tun_tap::{Iface, Mode};

type InterfaceHandle = Sender<InterfaceRequest>;

#[derive(Eq, Hash, PartialEq, Debug)]
pub struct Quad {
    // Quad: (SrcIp, SrcPort, DesIp, DesPort)
    src: (Ipv4Addr, u16),
    dst: (Ipv4Addr, u16),
}

enum InterfaceRequest {
    // NOTE: Read(number_of_bytes, channel_to_send_result_back)
    // NOTE: Write(data, channel_to_send_written_count_back)
    // NOTE: Flush(channel_to_confirm_done)
    Read {
        number_of_bytes: usize,
        ch_snd_res_back: Sender<Vec<u8>>,
    },
    Write {
        data: Vec<u8>,
        written_count: Sender<usize>,
    },
    Flush {
        ack: Sender<()>,
    },
    Bind {
        port: u16,
        ack: Sender<()>,
    },
    Unbind,
}

// Public API handle
pub struct Interface {
    tx: InterfaceHandle,
    jh: thread::JoinHandle<()>,
}

pub struct TcpListener {
    tx: InterfaceHandle,
}

impl Interface {
    pub fn new() -> io::Result<Self> {
        let connection_manger = ConnectionManager {
            nic: Iface::new("tun0", Mode::Tun).expect("Failed to create a TUN device"),
            // connections: Default::default(),
            buffer: vec![0u8; 1504],
        };
        let (tx, rx) = channel();
        let jh = thread::spawn(move || connection_manger.run(rx)); // NOTE: the run owns the mpsc queue
        Ok(Interface { tx, jh })
    }

    // NOTE: Connect and listen too??
    pub fn bind(&mut self, port: u16) -> io::Result<TcpListener> {
        // We create a temporary channel for communtication between Interface and connection_manger thread
        let (ack_tx, ack_rx) = channel();

        self.tx
            .send(InterfaceRequest::Bind { port, ack: ack_tx })
            .unwrap();
        ack_rx.recv().unwrap();

        Ok(TcpListener {
            tx: self.tx.clone(),
        })
    }
}

// Holds all the conn info for ALL OPEN CONN
// Kernel of our system
// owns the nic
struct ConnectionManager {
    nic: tun_tap::Iface,
    buffer: Vec<u8>,
    // connections: HashMap<Quad, Connection>,
}

impl ConnectionManager {
    // TODO: Create the TCPStream at the right time
    pub fn run(&self, rx: Receiver<InterfaceRequest>) {
        // Main Even Loop
        for reqeust in rx {}
    }
}

pub struct TcpStream {
    tx: InterfaceHandle,
    quad: Quad,
    state: Connection,
    incomming_buffer: Vec<u8>,
    outgoing_buffer: Vec<u8>,
    reassembly_buffer: BTreeMap<u32, Vec<u8>>,
}

impl Read for TcpStream {
    // Pull some bytes from this source into the specified buffer, returning how many bytes were read
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let (read_tx, read_rx) = channel();

        self.tx
            .send(InterfaceRequest::Read {
                number_of_bytes: buf.len(),
                ch_snd_res_back: read_tx,
            })
            .unwrap();

        // Say [A, B, C] => so the rx should come [A, B, C] saying that I recv that
        let recv_bytes = read_rx.recv().unwrap();
        buf[..recv_bytes.len()].copy_from_slice(&recv_bytes);
        Ok(recv_bytes.len())
    }
}

impl Write for TcpStream {
    // Writes a buffer into this writer, returning how many bytes were written
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        unimplemented!()
    }

    // flushes this output stream, ensuring that all intermediately buffered contents reach their destination
    fn flush(&mut self) -> io::Result<()> {
        unimplemented!()
    }
}
