use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use tokio::runtime::RngSeed;
use tokio::time::Duration;
use tokio::time::Instant;
use tokio::time::sleep;
use tracing::Level;

#[test]
fn test_deterministic_select_with_seed() {
    let seed = RngSeed::from_bytes(b"44");
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .start_paused(true)
        .rng_seed(seed)
        .build_local(&mut Default::default())
        .expect("could not bootstrap runtime");

    rt.block_on(async {
        let winner = Arc::new(AtomicUsize::new(0));

        let w1 = winner.clone();
        let f1 = async move {
            sleep(Duration::from_secs(1)).await;
            let _ = w1.compare_exchange(0, 1, Ordering::Relaxed, Ordering::Relaxed);
        };

        let w2 = winner.clone();
        let f2 = async move {
            sleep(Duration::from_secs(1)).await;
            let _ = w2.compare_exchange(0, 2, Ordering::Relaxed, Ordering::Relaxed);
        };

        tokio::time::advance(Duration::from_secs(1)).await;

        tokio::select! {
            _ = f1 => {},
            _ = f2 => {},
        }

        let result = winner.load(Ordering::Relaxed);
        match result {
            1 => println!("f1 won the select!"),
            2 => println!("f2 won the select!"),
            other => println!("Unexpected value: {}", other),
        }

        assert!(
            result == 1 || result == 2,
            "Expected a winner, got {}",
            result
        );
    });
}

#[tokio::test(start_paused = true)]
async fn auto_advance_kicks_in_when_idle() {
    // Configure tracing pour afficher les traces
    let _ = tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .try_init();

    let start = Instant::now();

    // Sleep for 5 seconds. Since the runtime is paused, you'd expect this to hang.
    // But Tokio auto-advances time because there's no other work.
    sleep(Duration::from_secs(5)).await;

    let elapsed = start.elapsed();

    // This will be exactly 5 seconds (simulated time)
    assert_eq!(elapsed, Duration::from_secs(5));

    println!("Elapsed: {:?}", elapsed);
}
