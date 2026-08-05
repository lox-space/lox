// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Synchronization primitives for long-running computations.

use alloc::sync::Arc;
use core::sync::atomic::{AtomicBool, Ordering};

/// A cooperative cancellation handle.
///
/// Cloning the token shares the underlying flag: cancelling any clone cancels
/// all of them. [`child`](Self::child) instead links two tokens *one way* —
/// see there for why both relationships are needed.
#[derive(Clone, Debug, Default)]
pub struct CancellationToken(Arc<Node>);

#[derive(Debug, Default)]
struct Node {
    cancelled: AtomicBool,
    parent: Option<CancellationToken>,
}

impl CancellationToken {
    /// Creates a new cancellation token.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a token that is cancelled when either it or `self` is cancelled,
    /// but whose own cancellation does **not** propagate back to `self`.
    ///
    /// This is what lets a component cancel its own work in response to a
    /// caller-supplied token without ever cancelling the caller's: a shared
    /// clone would propagate in both directions, so the component could not
    /// stop itself without stopping whatever else the caller attached to that
    /// token.
    pub fn child(&self) -> Self {
        Self(Arc::new(Node {
            cancelled: AtomicBool::new(false),
            parent: Some(self.clone()),
        }))
    }

    /// Sends cancellation.
    pub fn cancel(&self) {
        self.0.cancelled.store(true, Ordering::Relaxed);
    }

    /// Returns `true` if cancellation has been sent to this token or, when it is
    /// a [`child`](Self::child), to any of its ancestors.
    pub fn is_cancelled(&self) -> bool {
        self.0.cancelled.load(Ordering::Relaxed)
            || self
                .0
                .parent
                .as_ref()
                .is_some_and(CancellationToken::is_cancelled)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cancellation_token_is_shared() {
        let token = CancellationToken::new();
        let clone = token.clone();
        assert!(!token.is_cancelled());
        clone.cancel();
        assert!(token.is_cancelled());
        assert!(clone.is_cancelled());
    }

    #[test]
    fn test_child_observes_parent_cancellation() {
        let parent = CancellationToken::new();
        let child = parent.child();
        assert!(!child.is_cancelled());
        parent.cancel();
        assert!(child.is_cancelled());
    }

    #[test]
    fn test_child_cancellation_does_not_propagate_to_parent() {
        let parent = CancellationToken::new();
        let child = parent.child();
        child.cancel();
        assert!(child.is_cancelled());
        assert!(
            !parent.is_cancelled(),
            "child cancellation must not stop the caller's other work"
        );
    }

    #[test]
    fn test_child_chains_transitively() {
        let root = CancellationToken::new();
        let grandchild = root.child().child();
        root.cancel();
        assert!(grandchild.is_cancelled());
    }
}
