use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::sync::Arc;
use tokio::sync::watch;
use tokio::time::{interval_at, Instant, MissedTickBehavior};
use crate::Events::EventRunner::run_events;
const PERIOD: Duration = Duration::from_secs(60);
use crate::types::AppState;
/// Snapshot of one tick of the loop.
#[derive(Clone, Copy, Debug)]
pub struct TickInfo {
    /// 1 for the first tick, 2 for the second, ...
    pub round: u64,
    /// When this tick was scheduled to fire (monotonic clock).
    pub at: Instant,
    /// When the next tick is scheduled to fire.
    pub next_at: Instant,
    /// Wall-clock time of this tick, rounded down to the minute.
    pub unix_secs: i64,
}

/// Cheap, cloneable view into the running loop. Put it in your axum state.
#[derive(Clone)]
pub struct LoopHandle {
    rx: watch::Receiver<Option<TickInfo>>,
}

impl LoopHandle {
    /// The most recent tick, or None before the first one has fired.
    pub fn last_tick(&self) -> Option<TickInfo> {
        *self.rx.borrow()
    }

    /// How long until the loop's next tick.
    pub fn next_tick_in(&self) -> Duration {
        match self.last_tick() {
            Some(t) => t.next_at.saturating_duration_since(Instant::now()),
            None => until_next_minute(), // loop is still waiting for its first tick
        }
    }

    /// Wait for the next tick (returns None if the loop has stopped).
    pub async fn next_tick(&mut self) -> Option<TickInfo> {
        self.rx.changed().await.ok()?;
        *self.rx.borrow_and_update()
    }
}

fn unix_minute_now() -> i64 {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock is before 1970")
        .as_secs() as i64;
    secs - secs % 60
}

/// Starts a loop that fires on every whole minute and calls `on_tick` each time.
///
/// `on_tick` runs inside the loop task, so keep it quick: spawn the real work
/// (the Lambda invokes) with `tokio::spawn` so a slow region can't delay the
/// next tick.
pub fn spawn_loop<F>(mut on_tick: F, state: &Arc<AppState>) -> LoopHandle
where
    F: FnMut(TickInfo) + Send + 'static,
{
    let (tx, rx) = watch::channel(None);
    let state_clone = state.clone();
    tokio::spawn(async move {
        let start = Instant::now() + until_next_minute();
        let mut ticker = interval_at(start, PERIOD);
        ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);

        let mut round = 0u64;
        loop {
            let at = ticker.tick().await;
            round += 1;
            let info = TickInfo {
                round,
                at,
                next_at: at + PERIOD,
                unix_secs: unix_minute_now(),
            };
            let _ = tx.send(Some(info)); // publish state for anyone asking
            run_events(&state_clone);
        }
    });

    LoopHandle { rx }
}

fn until_next_minute() -> Duration {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock is before 1970");

    let seconds = now.as_secs();
    let nanos = now.subsec_nanos();

    let seconds_into_minute = seconds % 60;

    let remaining_seconds = 59 - seconds_into_minute;
    let remaining_nanos = 1_000_000_000 - nanos;

    Duration::from_secs(remaining_seconds)
        + Duration::from_nanos(remaining_nanos as u64)
}
