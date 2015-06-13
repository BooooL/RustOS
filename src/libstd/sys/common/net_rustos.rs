// TODO(ryan): implement the following for rust sockets:

use net::{SocketAddr, Shutdown, IpAddr};
use io::{self, Error, ErrorKind};
use fmt;
use sys_common::{AsInner, FromInner, IntoInner};
use prelude::v1::*;

pub struct Socket;

pub struct LookupHost;

impl Iterator for LookupHost {
    type Item = io::Result<SocketAddr>;
    fn next(&mut self) -> Option<io::Result<SocketAddr>> {
        unimplemented!();
    }
}

pub fn lookup_host(host: &str) -> io::Result<LookupHost> {
    unimplemented!();
}

////////////////////////////////////////////////////////////////////////////////
// lookup_addr
////////////////////////////////////////////////////////////////////////////////

pub fn lookup_addr(addr: &IpAddr) -> io::Result<String> {
    unimplemented!();
}

////////////////////////////////////////////////////////////////////////////////
// TCP streams
////////////////////////////////////////////////////////////////////////////////

pub struct TcpStream;

impl TcpStream {
    pub fn connect(addr: &SocketAddr) -> io::Result<TcpStream> {
        unimplemented!();
    }

    pub fn socket(&self) -> &Socket { unimplemented!(); }

    pub fn set_nodelay(&self, nodelay: bool) -> io::Result<()> {
        unimplemented!();
    }

    pub fn set_keepalive(&self, seconds: Option<u32>) -> io::Result<()> {
        unimplemented!();
    }

    fn set_tcp_keepalive(&self, seconds: u32) -> io::Result<()> {
        unimplemented!();
    }

    pub fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        unimplemented!();
    }

    pub fn write(&self, buf: &[u8]) -> io::Result<usize> {
        unimplemented!();
    }

    pub fn peer_addr(&self) -> io::Result<SocketAddr> {
        unimplemented!();
    }

    pub fn socket_addr(&self) -> io::Result<SocketAddr> {
        unimplemented!();
    }

    pub fn shutdown(&self, how: Shutdown) -> io::Result<()> {
        unimplemented!();
    }

    pub fn duplicate(&self) -> io::Result<TcpStream> {
        unimplemented!();
    }
}

impl FromInner<Socket> for TcpStream {
    fn from_inner(socket: Socket) -> TcpStream {
        unimplemented!();
    }
}

impl fmt::Debug for TcpStream {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unimplemented!();
    }
}

////////////////////////////////////////////////////////////////////////////////
// TCP listeners
////////////////////////////////////////////////////////////////////////////////

pub struct TcpListener;

impl TcpListener {
    pub fn bind(addr: &SocketAddr) -> io::Result<TcpListener> {
        unimplemented!();
    }

    pub fn socket(&self) -> &Socket { unimplemented!(); }

    pub fn socket_addr(&self) -> io::Result<SocketAddr> {
        unimplemented!();
    }

    pub fn accept(&self) -> io::Result<(TcpStream, SocketAddr)> {
        unimplemented!();
    }

    pub fn duplicate(&self) -> io::Result<TcpListener> {
        unimplemented!();
    }
}

impl FromInner<Socket> for TcpListener {
    fn from_inner(socket: Socket) -> TcpListener {
        unimplemented!();
    }
}

impl fmt::Debug for TcpListener {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unimplemented!();
    }
}

////////////////////////////////////////////////////////////////////////////////
// UDP
////////////////////////////////////////////////////////////////////////////////

pub struct UdpSocket;

impl UdpSocket {
    pub fn bind(addr: &SocketAddr) -> io::Result<UdpSocket> {
        unimplemented!();
    }

    pub fn socket(&self) -> &Socket { unimplemented!(); }

    pub fn socket_addr(&self) -> io::Result<SocketAddr> {
        unimplemented!();
    }

    pub fn recv_from(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        unimplemented!();
    }

    pub fn send_to(&self, buf: &[u8], dst: &SocketAddr) -> io::Result<usize> {
        unimplemented!();
    }

    pub fn set_broadcast(&self, on: bool) -> io::Result<()> {
        unimplemented!();
    }

    pub fn set_multicast_loop(&self, on: bool) -> io::Result<()> {
        unimplemented!();
    }

    pub fn join_multicast(&self, multi: &IpAddr) -> io::Result<()> {
        unimplemented!();
    }
    pub fn leave_multicast(&self, multi: &IpAddr) -> io::Result<()> {
        unimplemented!();
    }
    pub fn multicast_time_to_live(&self, ttl: i32) -> io::Result<()> {
        unimplemented!();
    }

    pub fn time_to_live(&self, ttl: i32) -> io::Result<()> {
        unimplemented!();
    }

    pub fn duplicate(&self) -> io::Result<UdpSocket> {
        unimplemented!();
    }
}

impl FromInner<Socket> for UdpSocket {
    fn from_inner(socket: Socket) -> UdpSocket {
        unimplemented!();
    }
}

impl fmt::Debug for UdpSocket {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unimplemented!();
    }
}