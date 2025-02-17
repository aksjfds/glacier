use std::{
    future::Future,
    net::SocketAddr,
    pin::Pin,
    task::{Context, Poll},
};

use tokio::net::{TcpListener, TcpStream};

pub struct PollStream {
    listener: TcpListener,
    capacity: usize,
}

impl PollStream {
    pub fn with_capacity(capacity: usize, listener: TcpListener) -> Self {
        PollStream { listener, capacity }
    }

    pub async fn poll_some(&mut self) -> Vec<Result<(TcpStream, SocketAddr), std::io::Error>> {
        unsafe { Pin::new_unchecked(self).await }
    }
}

impl Future for PollStream {
    type Output = Vec<Result<(TcpStream, SocketAddr), std::io::Error>>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut streams = Vec::with_capacity(self.capacity);
        while let Poll::Ready(stream) = self.listener.poll_accept(cx) {
            streams.push(stream);
            if streams.len() >= self.capacity {
                return Poll::Ready(streams);
            }
        }

        if streams.len() == 0 {
            Poll::Pending
        } else {
            Poll::Ready(streams)
        }
    }
}
