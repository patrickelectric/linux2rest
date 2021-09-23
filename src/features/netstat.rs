use netstat2::{AddressFamilyFlags, ProtocolFlags, ProtocolSocketInfo};
use paperclip::actix::Apiv2Schema;
use serde::Serialize;

#[derive(Debug, Serialize, Apiv2Schema)]
struct AddressPort {
    address: String,
    port: u16,
}

#[derive(Debug, Serialize, Apiv2Schema)]
struct Udp {
    local: AddressPort,
    pids: Vec<u32>,
}

impl Udp {
    fn new(udp: &netstat2::UdpSocketInfo, socket_info: &netstat2::SocketInfo) -> Udp {
        Udp {
            local: AddressPort {
                address: udp.local_addr.to_string(),
                port: udp.local_port,
            },
            pids: socket_info.associated_pids.clone(),
        }
    }
}

#[derive(Debug, Serialize, Apiv2Schema)]
struct Tcp {
    local: AddressPort,
    remote: AddressPort,
    pids: Vec<u32>,
    state: String,
}

impl Tcp {
    fn new(tcp: &netstat2::TcpSocketInfo, socket_info: &netstat2::SocketInfo) -> Tcp {
        Tcp {
            local: AddressPort {
                address: tcp.local_addr.to_string(),
                port: tcp.local_port,
            },
            remote: AddressPort {
                address: tcp.remote_addr.to_string(),
                port: tcp.remote_port,
            },
            pids: socket_info.associated_pids.clone(),
            state: format!("{}", tcp.state),
        }
    }
}

#[derive(Debug, Serialize, Apiv2Schema)]
pub struct Netstat {
    tcp: Vec<Tcp>,
    udp: Vec<Udp>,
}

pub fn netstat() -> Netstat {
    let sockets_info_iter = netstat2::get_sockets_info(
        AddressFamilyFlags::IPV4,
        ProtocolFlags::TCP | ProtocolFlags::UDP,
    )
    .unwrap();

    let tcps = sockets_info_iter
        .iter()
        .filter_map(|socket_info| match &socket_info.protocol_socket_info {
            ProtocolSocketInfo::Tcp(tcp) => Some((socket_info.clone(), tcp)),
            ProtocolSocketInfo::Udp(_udp) => None,
        })
        .map(|(socket_info, tcp)| Tcp::new(tcp, &socket_info))
        .collect::<Vec<Tcp>>();

    let udps = sockets_info_iter
        .iter()
        .filter_map(|socket_info| match &socket_info.protocol_socket_info {
            ProtocolSocketInfo::Tcp(_tcp) => None,
            ProtocolSocketInfo::Udp(udp) => Some((socket_info.clone(), udp)),
        })
        .map(|(socket_info, udp)| Udp::new(udp, &socket_info))
        .collect::<Vec<Udp>>();

    Netstat {
        tcp: tcps,
        udp: udps,
    }
}
