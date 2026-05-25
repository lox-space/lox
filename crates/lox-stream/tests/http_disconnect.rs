// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use axum::Router;
use axum::extract::State;
use axum::response::sse::{Event, Sse};
use axum::routing::get;
use futures::StreamExt;
use lox_stream::{OnError, par_stream};

#[derive(Clone)]
struct AppState {
    counter: Arc<AtomicUsize>,
}

async fn stream_handler(
    State(state): State<AppState>,
) -> Sse<impl futures::Stream<Item = Result<Event, std::convert::Infallible>>> {
    let counter = state.counter.clone();
    let s = par_stream(0..10_000_usize, 16, OnError::Continue, move |i, _| {
        counter.fetch_add(1, Ordering::SeqCst);
        std::thread::sleep(Duration::from_millis(50));
        Ok::<usize, ()>(i)
    });
    Sse::new(s.map(|r| Ok(Event::default().data(format!("{}", r.unwrap())))))
}

#[tokio::test]
#[ignore = "integration; flaky under CI thread contention"]
async fn http_disconnect_cancels_work() {
    let counter = Arc::new(AtomicUsize::new(0));
    let state = AppState {
        counter: counter.clone(),
    };
    let app = Router::new()
        .route("/stream", get(stream_handler))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let client = reqwest::Client::new();
    let resp = client
        .get(format!("http://{addr}/stream"))
        .send()
        .await
        .unwrap();
    let mut bytes = resp.bytes_stream();
    bytes.next().await; // receive one chunk
    drop(bytes); // simulate browser disconnect

    tokio::time::sleep(Duration::from_millis(500)).await;
    let after_disconnect = counter.load(Ordering::SeqCst);
    tokio::time::sleep(Duration::from_secs(1)).await;
    let later = counter.load(Ordering::SeqCst);

    // After disconnect, growth should be bounded to roughly worker_count.
    let max_growth = rayon::current_num_threads() * 2;
    assert!(
        later - after_disconnect <= max_growth,
        "growth {} > {} after disconnect",
        later - after_disconnect,
        max_growth,
    );
}
