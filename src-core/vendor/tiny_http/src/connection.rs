//! Abstractions of Tcp and Unix socket types

#[cfg(unix)]
use std::os::unix::net as unix_net;
use std::{
    net::{Shutdown, SocketAddr, TcpListener, TcpStream, ToSocketAddrs},
    path::PathBuf,
};

/// Unified listener. Either a [`TcpListener`] or [`std::os::unix::net::UnixListener`]
pub enum Listener {
    Tcp(TcpListener),
    #[cfg(unix)]
    Unix(unix_net::UnixListener),
}
impl Listener {
    pub(crate) fn local_addr(&self) -> std::io::Result<ListenAddr> {
        match self {
            Self::Tcp(l) => l.local_addr().map(ListenAddr::from),
            #[cfg(unix)]
            Self::Unix(l) => l.local_addr().map(ListenAddr::from),
        }
    }

    pub(crate) fn accept(&self) -> std::io::Result<(Connection, Option<SocketAddr>)> {
        match self {
            Self::Tcp(l) => l
                .accept()
                .map(|(conn, addr)| (Connection::from(conn), Some(addr))),
            #[cfg(unix)]
            Self::Unix(l) => l.accept().map(|(conn, _)| (Connection::from(conn), None)),
        }
    }
}
impl From<TcpListener> for Listener {
    fn from(s: TcpListener) -> Self {
        Self::Tcp(s)
    }
}
#[cfg(unix)]
impl From<unix_net::UnixListener> for Listener {
    fn from(s: unix_net::UnixListener) -> Self {
        Self::Unix(s)
    }
}

/// Unified connection. Either a [`TcpStream`] or [`std::os::unix::net::UnixStream`].
#[derive(Debug)]
pub(crate) enum Connection {
    Tcp(DeadlineTcpStream),
    #[cfg(unix)]
    Unix(unix_net::UnixStream),
}
impl std::io::Read for Connection {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Self::Tcp(s) => s.read(buf),
            #[cfg(unix)]
            Self::Unix(s) => s.read(buf),
        }
    }
}
impl std::io::Write for Connection {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            Self::Tcp(s) => s.write(buf),
            #[cfg(unix)]
            Self::Unix(s) => s.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Self::Tcp(s) => s.flush(),
            #[cfg(unix)]
            Self::Unix(s) => s.flush(),
        }
    }
}
impl Connection {
    // The permit follows every socket clone and queued Request, not merely the
    // accept task. Refuse overload before a worker or request can be allocated.
    pub(crate) fn admit(&mut self, count: &std::sync::Arc<std::sync::atomic::AtomicUsize>) -> bool {
        match self {
            Self::Tcp(s) => {
                if count
                    .fetch_update(
                        std::sync::atomic::Ordering::AcqRel,
                        std::sync::atomic::Ordering::Acquire,
                        |n| (n < 32).then_some(n + 1),
                    )
                    .is_err()
                {
                    return false;
                }
                s.permit = Some(std::sync::Arc::new(ConnectionPermit(count.clone())));
                true
            }
            #[cfg(unix)]
            Self::Unix(_) => true,
        }
    }

    /// Gets the peer's address. Some for TCP, None for Unix sockets.
    pub(crate) fn peer_addr(&mut self) -> std::io::Result<Option<SocketAddr>> {
        match self {
            Self::Tcp(s) => s.peer_addr().map(Some),
            #[cfg(unix)]
            Self::Unix(_) => Ok(None),
        }
    }

    pub(crate) fn shutdown(&self, how: Shutdown) -> std::io::Result<()> {
        match self {
            Self::Tcp(s) => s.shutdown(how),
            #[cfg(unix)]
            Self::Unix(s) => s.shutdown(how),
        }
    }

    pub(crate) fn try_clone(&self) -> std::io::Result<Self> {
        match self {
            Self::Tcp(s) => s.try_clone().map(Self::Tcp),
            #[cfg(unix)]
            Self::Unix(s) => s.try_clone().map(Self::from),
        }
    }
}
impl From<TcpStream> for Connection {
    fn from(s: TcpStream) -> Self {
        Self::Tcp(DeadlineTcpStream::new(s))
    }
}
#[cfg(unix)]
impl From<unix_net::UnixStream> for Connection {
    fn from(s: unix_net::UnixStream) -> Self {
        Self::Unix(s)
    }
}

#[derive(Debug, Clone)]
pub enum ConfigListenAddr {
    IP(Vec<SocketAddr>),
    #[cfg(unix)]
    // TODO: use SocketAddr when bind_addr is stabilized
    Unix(std::path::PathBuf),
}
impl ConfigListenAddr {
    pub fn from_socket_addrs<A: ToSocketAddrs>(addrs: A) -> std::io::Result<Self> {
        addrs.to_socket_addrs().map(|it| Self::IP(it.collect()))
    }

    #[cfg(unix)]
    pub fn unix_from_path<P: Into<PathBuf>>(path: P) -> Self {
        Self::Unix(path.into())
    }

    pub(crate) fn bind(&self) -> std::io::Result<Listener> {
        match self {
            Self::IP(a) => TcpListener::bind(a.as_slice()).map(Listener::from),
            #[cfg(unix)]
            Self::Unix(a) => unix_net::UnixListener::bind(a).map(Listener::from),
        }
    }
}

/// Unified listen socket address. Either a [`SocketAddr`] or [`std::os::unix::net::SocketAddr`].
#[derive(Debug, Clone)]
pub enum ListenAddr {
    IP(SocketAddr),
    #[cfg(unix)]
    Unix(unix_net::SocketAddr),
}
impl ListenAddr {
    pub fn to_ip(self) -> Option<SocketAddr> {
        match self {
            Self::IP(s) => Some(s),
            #[cfg(unix)]
            Self::Unix(_) => None,
        }
    }

    /// Gets the Unix socket address.
    ///
    /// This is also available on non-Unix platforms, for ease of use, but always returns `None`.
    #[cfg(unix)]
    pub fn to_unix(self) -> Option<unix_net::SocketAddr> {
        match self {
            Self::IP(_) => None,
            Self::Unix(s) => Some(s),
        }
    }
    #[cfg(not(unix))]
    pub fn to_unix(self) -> Option<SocketAddr> {
        None
    }
}
impl From<SocketAddr> for ListenAddr {
    fn from(s: SocketAddr) -> Self {
        Self::IP(s)
    }
}
#[cfg(unix)]
impl From<unix_net::SocketAddr> for ListenAddr {
    fn from(s: unix_net::SocketAddr) -> Self {
        Self::Unix(s)
    }
}
impl std::fmt::Display for ListenAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IP(s) => s.fmt(f),
            #[cfg(unix)]
            Self::Unix(s) => std::fmt::Debug::fmt(s, f),
        }
    }
}

// Aegis transport policy: an absolute ingress deadline, including headers, bodies,
// keep-alive and rejected-body draining. A slow trickle cannot renew the budget.
#[derive(Debug)]
pub(crate) struct DeadlineTcpStream {
    socket: TcpStream,
    permit: Option<std::sync::Arc<ConnectionPermit>>,
    read_deadline: std::time::Instant,
    write_deadline: std::sync::Arc<std::sync::Mutex<Option<std::time::Instant>>>,
}
impl DeadlineTcpStream {
    fn new(socket: TcpStream) -> Self {
        Self {
            socket,
            permit: None,
            read_deadline: std::time::Instant::now() + std::time::Duration::from_secs(5),
            write_deadline: std::sync::Arc::new(std::sync::Mutex::new(None)),
        }
    }
    fn remaining(deadline: std::time::Instant) -> std::io::Result<std::time::Duration> {
        deadline
            .checked_duration_since(std::time::Instant::now())
            .filter(|d| !d.is_zero())
            .ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "Aegis transport deadline exceeded",
                )
            })
    }
    fn try_clone(&self) -> std::io::Result<Self> {
        Ok(Self {
            socket: self.socket.try_clone()?,
            permit: self.permit.clone(),
            read_deadline: self.read_deadline,
            write_deadline: self.write_deadline.clone(),
        })
    }
}
impl std::ops::Deref for DeadlineTcpStream {
    type Target = TcpStream;
    fn deref(&self) -> &TcpStream {
        &self.socket
    }
}
impl std::io::Read for DeadlineTcpStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.socket
            .set_read_timeout(Some(Self::remaining(self.read_deadline)?))?;
        std::io::Read::read(&mut self.socket, buf)
    }
}
impl std::io::Write for DeadlineTcpStream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let deadline = {
            let mut deadline = self
                .write_deadline
                .lock()
                .map_err(|_| std::io::Error::other("Transport deadline state unavailable"))?;
            *deadline.get_or_insert_with(|| {
                std::time::Instant::now() + std::time::Duration::from_secs(5)
            })
        };
        self.socket
            .set_write_timeout(Some(Self::remaining(deadline)?))?;
        std::io::Write::write(&mut self.socket, buf)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        std::io::Write::flush(&mut self.socket)
    }
}

#[derive(Debug)]
struct ConnectionPermit(std::sync::Arc<std::sync::atomic::AtomicUsize>);
impl Drop for ConnectionPermit {
    fn drop(&mut self) {
        self.0.fetch_sub(1, std::sync::atomic::Ordering::AcqRel);
    }
}
