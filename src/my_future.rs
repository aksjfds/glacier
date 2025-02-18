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

pub struct MyFuture<F: Future<Output = ()>> {
    f: F,
    done: bool,
}

impl<F: Future<Output = ()>> MyFuture<F> {
    pub fn new(f: F) -> Self {
        MyFuture { f, done: false }
    }

    pub fn is_done(&self) -> bool {
        self.done
    }
}

impl<F: Future<Output = ()>> Future for MyFuture<F> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let my_future = unsafe { self.get_unchecked_mut() };
        match my_future {
            MyFuture { f, done: false } => {
                let f = unsafe { Pin::new_unchecked(f) };
                match f.poll(cx) {
                    Poll::Ready(_) => {
                        my_future.done = true;
                        Poll::Ready(())
                    }
                    Poll::Pending => Poll::Pending,
                }
            }
            _ => Poll::Ready(()),
        }
    }
}

pub struct MyFutureTasks<F: Future<Output = ()>> {
    tasks: Vec<MyFuture<F>>,
}

impl<F: Future<Output = ()>> MyFutureTasks<F> {
    pub fn new(tasks: Vec<MyFuture<F>>) -> Self {
        MyFutureTasks { tasks }
    }
}

impl<F: Future<Output = ()>> Future for MyFutureTasks<F> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let my_future_tasks = unsafe { self.get_unchecked_mut() };

        let mut all_done = true;
        my_future_tasks.tasks.iter_mut().for_each(|f| {
            if !f.is_done() {
                let f = unsafe { Pin::new_unchecked(f) };
                if f.poll(cx).is_pending() {
                    all_done = false;
                }
            }
        });

        if all_done {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}
