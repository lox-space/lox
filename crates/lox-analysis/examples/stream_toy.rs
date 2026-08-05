// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Streams twenty toy targets to stdout so the engine's behaviour is visible
//! rather than only asserted.
//!
//! ```text
//! cargo run -p lox-analysis --features async --example stream_toy
//! ```
//!
//! What to look for:
//!
//! - **Interleaving.** Targets emit at different rates, so lines from different
//!   targets alternate instead of arriving target-by-target. Nothing about the
//!   output order tells you which target a line belongs to — the key does.
//! - **Back-pressure.** The consumer deliberately sleeps every few items. The
//!   producers cannot run away: total items in flight stays bounded by the
//!   channel, so the run paces itself to the reader.
//! - **`Completed` exactly once per target**, last for that target, including
//!   for target 7, which fails partway through, and target 13, which produces
//!   nothing at all.
//! - **Cancellation.** Ctrl-C, and the workers stop: the summary prints
//!   immediately and the process exits without a lingering busy core. The
//!   targets that had not finished report no `Completed`.

use std::collections::BTreeMap;
use std::time::Duration;

use futures_util::StreamExt;
use lox_analysis::pipeline::AnalysisError;
use lox_analysis::stream::{StreamEvent, stream};
use lox_core::error::LoxError;
use lox_core::sync::CancellationToken;

#[derive(Debug, thiserror::Error)]
#[error("target {0} hit a simulated failure")]
struct SimulatedFailure(usize);

/// A toy scan: `n` items, `delay` between them, optionally failing at `fail_at`.
///
/// The sleep stands in for a real detector sweep, and the cancellation check
/// stands in for a source checking at its sampling boundaries — which is the
/// whole reason the token is handed to `make` rather than only being observed
/// between items.
fn toy_scan(
    id: usize,
    n: usize,
    delay: Duration,
    fail_at: Option<usize>,
    cancel: CancellationToken,
) -> impl Iterator<Item = Result<String, AnalysisError>> + Send + 'static {
    (0..n).map_while(move |i| {
        if cancel.is_cancelled() {
            return None;
        }
        std::thread::sleep(delay);
        if fail_at == Some(i) {
            return Some(Err(AnalysisError::Stage(LoxError::new(SimulatedFailure(
                id,
            )))));
        }
        Some(Ok(format!("item {i}")))
    })
}

#[tokio::main]
async fn main() {
    const TARGETS: usize = 20;

    let cancel = CancellationToken::new();
    let on_ctrl_c = cancel.clone();
    tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            eprintln!("\n-- Ctrl-C: cancelling --");
            on_ctrl_c.cancel();
        }
    });

    // Target 13 yields nothing; target 7 fails on its third item; the rest run
    // at staggered rates so the output interleaves.
    let inputs = (0..TARGETS).map(|id| {
        let n = if id == 13 { 0 } else { 6 + id % 5 };
        let fail_at = (id == 7).then_some(2);
        (
            id,
            (id, n, Duration::from_millis(20 + 7 * id as u64), fail_at),
        )
    });

    // The key is not visible inside `make` — only the input is — so anything a
    // pipeline needs to identify itself has to travel in the input.
    let mut events = std::pin::pin!(stream(
        inputs,
        |(id, n, delay, fail_at), cancel| toy_scan(id, n, delay, fail_at, cancel),
        Some(cancel),
    ));

    let mut items: BTreeMap<usize, usize> = BTreeMap::new();
    let mut errors: BTreeMap<usize, usize> = BTreeMap::new();
    let mut completed: BTreeMap<usize, usize> = BTreeMap::new();
    let mut seen = 0usize;

    println!("streaming {TARGETS} targets (Ctrl-C to cancel)\n");
    while let Some((key, event)) = events.next().await {
        match event {
            StreamEvent::Item(Ok(item)) => {
                *items.entry(key).or_default() += 1;
                println!("  target {key:>2}  {item}");
            }
            StreamEvent::Item(Err(e)) => {
                *errors.entry(key).or_default() += 1;
                println!("  target {key:>2}  ERROR: {e}");
            }
            StreamEvent::Completed => {
                *completed.entry(key).or_default() += 1;
                println!("  target {key:>2}  <completed>");
            }
        }

        // A deliberately slow consumer, so back-pressure is the thing pacing
        // the run rather than the producers' delays.
        seen += 1;
        if seen.is_multiple_of(5) {
            tokio::time::sleep(Duration::from_millis(40)).await;
        }
    }

    println!("\n{:->56}", "");
    println!(
        "{:>8}  {:>6}  {:>7}  {:>10}  note",
        "target", "items", "errors", "completed"
    );
    for id in 0..TARGETS {
        let c = completed.get(&id).copied().unwrap_or(0);
        let note = match (c, items.get(&id).copied().unwrap_or(0)) {
            (0, _) => "cancelled mid-flight or never started",
            (1, 0) => "completed with no items (normal)",
            (1, _) => "",
            _ => "BUG: more than one Completed",
        };
        println!(
            "{id:>8}  {:>6}  {:>7}  {:>10}  {note}",
            items.get(&id).copied().unwrap_or(0),
            errors.get(&id).copied().unwrap_or(0),
            c,
        );
    }

    let multiples: Vec<_> = completed
        .iter()
        .filter(|&(_, &c)| c != 1)
        .map(|(k, _)| *k)
        .collect();
    assert!(
        multiples.is_empty(),
        "the one-Completed-per-target invariant broke for {multiples:?}"
    );
    println!(
        "\n{} of {TARGETS} targets completed; {} produced an error item",
        completed.len(),
        errors.len()
    );
}
