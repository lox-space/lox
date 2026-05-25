// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use std::pin::Pin;
use std::task::{Context, Poll};

use crate::cancellation::CancellationToken;

/// A stream of `Result<T, E>` items produced by a parallel pipeline.
///
/// Implements [`futures_core::Stream`] for async consumers; also provides
/// [`Stream::blocking_next`] for sync callers.
pub struct Stream<T, E> {
    pub(crate) rx: async_channel::Receiver<Result<T, E>>,
    pub(crate) token: CancellationToken,
}

impl<T, E> Stream<T, E> {
    /// Returns a clone of the cancellation token associated with this stream.
    pub fn token(&self) -> CancellationToken {
        self.token.clone()
    }

    /// Flips the cancellation flag. Workers will bail at their next unit
    /// boundary. Items already in flight may still be delivered.
    pub fn cancel(&self) {
        self.token.cancel();
    }

    /// Synchronous pull. Blocks the current thread until the next item is
    /// available or the stream is exhausted.
    pub fn blocking_next(&mut self) -> Option<Result<T, E>> {
        self.rx.recv_blocking().ok()
    }
}

impl<T, E> futures_core::Stream for Stream<T, E> {
    type Item = Result<T, E>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        // SAFETY: we never move `this.rx`. The transient pin obtained here
        // lives only for this poll call, satisfying Receiver's pin contract.
        unsafe { Pin::new_unchecked(&mut this.rx) }.poll_next(cx)
    }
}

// Stream treats `rx` as a non-structural field: we never move it after
// construction, and `poll_next` only re-pins it transiently via
// `Pin::new_unchecked`. Drop does not move `rx`. Therefore it is sound to
// expose Stream as Unpin regardless of whether Receiver is.
impl<T, E> Unpin for Stream<T, E> {}

impl<T, E> Drop for Stream<T, E> {
    fn drop(&mut self) {
        self.token.cancel();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_stream<T, E>() -> (async_channel::Sender<Result<T, E>>, Stream<T, E>) {
        let (tx, rx) = async_channel::bounded(8);
        let token = CancellationToken::new();
        (tx, Stream { rx, token })
    }

    #[test]
    fn blocking_next_returns_ok_items() {
        let (tx, mut s) = make_stream::<i32, ()>();
        tx.send_blocking(Ok(1)).unwrap();
        tx.send_blocking(Ok(2)).unwrap();
        drop(tx);
        assert_eq!(s.blocking_next(), Some(Ok(1)));
        assert_eq!(s.blocking_next(), Some(Ok(2)));
        assert_eq!(s.blocking_next(), None);
    }

    #[test]
    fn cancel_flips_the_token() {
        let (_tx, s) = make_stream::<i32, ()>();
        let t = s.token();
        assert!(!t.is_cancelled());
        s.cancel();
        assert!(t.is_cancelled());
    }

    #[test]
    fn drop_flips_the_token() {
        let (_tx, s) = make_stream::<i32, ()>();
        let t = s.token();
        drop(s);
        assert!(t.is_cancelled());
    }

    #[tokio::test]
    async fn async_stream_delivers_in_order() {
        use futures::StreamExt;

        let (tx, mut s) = make_stream::<i32, ()>();
        tokio::spawn(async move {
            tx.send(Ok(10)).await.unwrap();
            tx.send(Ok(20)).await.unwrap();
            tx.send(Ok(30)).await.unwrap();
            drop(tx);
        });

        let mut collected = Vec::new();
        while let Some(item) = s.next().await {
            collected.push(item.unwrap());
        }
        assert_eq!(collected, vec![10, 20, 30]);
    }
}
