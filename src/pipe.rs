use futures_util::stream::Stream;
use parking_lot::Mutex;
/// Pipe is a stream which can be used to pipe data
/// from one end and read it from the other end.
/// this is a lot like the 'pipe' method in Node.js
/// # Attention
/// Avoid using this if you can has it is not very efficient
/// and resource intensive.
use std::{
    pin::Pin,
    sync::Arc,
    task::{Context, Poll, Waker},
};

/// this creates a two stream that can write from one end and read from the other
/// when 'clone' is called a new reference to the underlying stream is returned
/// allowing the two streams to be used in parallel, perhaps on different threads
/// # Note
/// Avoid using this if you can it is not very efficient
/// and resource intensive.
pub struct Pipe<T>(Arc<Mutex<TwoWayStream<T>>>);

impl<T> Default for Pipe<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Pipe<T> {
    /// Create a new `Pipe`.
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(TwoWayStream::new())))
    }

    pub fn send(&mut self, data: T) {
        let mut lock = self.0.lock();
        lock.data = Some(data);
        (*lock).wake_up_reader();
    }

    pub fn finish(&mut self) {
        let mut lock = self.0.lock();
        (*lock).finish();
    }
}

impl<T> Clone for Pipe<T> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<T: Unpin> Stream for Pipe<T> {
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let mut lock = self.0.lock();
        Pin::new(&mut *lock).poll_next(cx)
    }
}

#[derive(Debug, Clone)]
struct TwoWayStream<T> {
    data: Option<T>,
    is_finished: bool,
    reader_waker: Option<Waker>,
}

impl<T> TwoWayStream<T> {
    /// creates a new two way stream with the given capacity
    /// # Arguments
    /// * `capacity` - the capacity of the buffer
    /// # Panics
    /// Panics if the capacity is less than 0
    pub fn new() -> Self {
        Self {
            data: None,
            is_finished: false,
            reader_waker: None,
        }
    }

    fn wake_up_reader(&self) {
        if let Some(waker) = &self.reader_waker {
            waker.wake_by_ref();
        }
    }

    // this would flush the buffer to the underlying stream
    // this should only be called when the two stream is closed
    fn finish(&mut self) {
        self.is_finished = true;
    }
}

impl<T: Unpin> Stream for TwoWayStream<T> {
    type Item = T;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        if self.reader_waker.is_none() {
            self.reader_waker = Some(cx.waker().clone());
        }

        if self.data.is_none() {
            // this marks the end of the stream
            if self.is_finished {
                return Poll::Ready(None);
            }

            return Poll::Pending;
        }

        // we have some data in the buffer
        // the following should always  return Poll::Ready(Some(_))
        // because if it returns Poll::Ready(None) then the stream is finished which
        // isn't true
        Poll::Ready(self.data.take())
    }
}
