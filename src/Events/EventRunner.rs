//the point of this is to run any faas that are available
use crate::types::{Pokers, Db, AppState};
use std::sync::Arc;
use crate::Events::aws;
///Immediately spawning an async task bc this should mainly be async
pub fn run_events(state: &Arc<AppState>) {
    let state_arc = *state.clone();
    tokio::spawn(async move {
        if state_arc.pokers.aws_lambda {
            ///each event should run conccurrently
            spawn_task(aws::run_job, state_arc.db.clone());
        }
    });
}
#[inline]
fn spawn_task(event_task: fn (Db), db: Arc<Db>) {
    tokio::spawn(async move {
            event_task(db.clone());
    });
}