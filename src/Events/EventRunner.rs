//the point of this is to run any faas that are available
use crate::Events::aws;
use crate::types::{AppState, Db, Pokers};
use std::sync::Arc;
///Immediately spawning an async task bc this should mainly be async
pub fn run_events(state: &Arc<AppState>) {
    let state_arc = state.clone();
    tokio::spawn(async move {
        if state_arc.pokers.aws_lambda {
            ///each event should run conccurrently
            spawn_task(aws::run_job, state_arc.db.clone());
        }
    });
}
fn spawn_task<F, Fut>(event_task: F, db: Db)
where
    F: FnOnce(Db) -> Fut + Send + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    tokio::spawn(async move {
        event_task(db).await;
    });
}
