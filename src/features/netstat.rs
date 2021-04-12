use netstat2::{AddressFamilyFlags, ProtocolFlags, ProtocolSocketInfo};

pub fn generate_serde_value() -> serde_json::Value {
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
        .map(|(socket_info, tcp)| generate_serde_from_tcp(tcp, &socket_info))
        .collect::<Vec<serde_json::Value>>();

    let udps = sockets_info_iter
        .iter()
        .filter_map(|socket_info| match &socket_info.protocol_socket_info {
            ProtocolSocketInfo::Tcp(_tcp) => None,
            ProtocolSocketInfo::Udp(udp) => Some((socket_info.clone(), udp)),
        })
        .map(|(socket_info, udp)| generate_serde_from_udp(udp, &socket_info))
        .collect::<Vec<serde_json::Value>>();

    serde_json::json!({
        "tcp": tcps,
        "udp": udps,
    })
}

fn generate_serde_from_tcp(
    tcp: &netstat2::TcpSocketInfo,
    socket_info: &netstat2::SocketInfo,
) -> serde_json::Value {
    serde_json::json!({
        "local": {
            "address": tcp.local_addr,
            "port": tcp.local_port,
        },
        "remote": {
            "address": tcp.remote_addr,
            "port": tcp.remote_port,
        },
        "pids": socket_info.associated_pids,
        "state": format!("{}", tcp.state),
    })
}

fn generate_serde_from_udp(
    tcp: &netstat2::UdpSocketInfo,
    socket_info: &netstat2::SocketInfo,
) -> serde_json::Value {
    serde_json::json!({
        "local": {
            "address": tcp.local_addr,
            "port": tcp.local_port,
        },
        "pids": socket_info.associated_pids,
    })
}
