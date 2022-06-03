use std::{
    fmt::Debug,
    future::Future,
    io::{self, IoSliceMut},
    net::SocketAddr,
    pin::Pin,
    task::{Context, Poll},
    time::Instant,
};

use proto::Transmit;
use udp::{RecvMeta, UdpState};

/// Abstracts I/O and timer operations for runtime independence
pub trait Runtime: Send + Sync + Debug + 'static {
    fn new_timer(&self, i: Instant) -> Pin<Box<dyn AsyncTimer>>;
    fn spawn(&self, future: Pin<Box<dyn Future<Output = ()> + Send>>);
    fn wrap_udp_socket(&self, t: std::net::UdpSocket) -> io::Result<Box<dyn AsyncUdpSocket>>;
}

/// Abstract implementation of an async timer for runtime independence
pub trait AsyncTimer: Send + Debug {
    fn reset(self: Pin<&mut Self>, i: Instant);
    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<()>;
}

/// Abstract implementation of a UDP socket for runtime independence
pub trait AsyncUdpSocket: Send + Debug {
    fn poll_send(
        &mut self,
        state: &UdpState,
        cx: &mut Context,
        transmits: &[Transmit],
    ) -> Poll<Result<usize, io::Error>>;

    fn poll_recv(
        &self,
        cx: &mut Context,
        bufs: &mut [IoSliceMut<'_>],
        meta: &mut [RecvMeta],
    ) -> Poll<io::Result<usize>>;

    fn local_addr(&self) -> io::Result<SocketAddr>;
}

/// Automatically select an appropriate runtime from those enabled at compile time
pub fn default_runtime() -> Option<Box<dyn Runtime>> {
    #[cfg(feature = "runtime-tokio")]
    {
        if ::tokio::runtime::Handle::try_current().is_ok() {
            return Some(Box::new(TokioRuntime));
        }
    }

    #[cfg(feature = "runtime-async-std")]
    {
        return Some(Box::new(AsyncStdRuntime));
    }

    #[cfg(not(feature = "runtime-async-std"))]
    None
}

#[cfg(feature = "runtime-tokio")]
mod tokio;
#[cfg(feature = "runtime-tokio")]
pub use self::tokio::TokioRuntime;

#[cfg(feature = "runtime-async-std")]
mod async_std;
#[cfg(feature = "runtime-async-std")]
pub use self::async_std::AsyncStdRuntime;
