pub async fn poller(db: Db) {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .unwrap();

    let mut tick = interval(Duration::from_secs(60));
    tick.set_missed_tick_behavior(MissedTickBehavior::Skip);

    loop {
        tick.tick().await;
        let r = ping(&client).await;
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let conn = db.lock().unwrap();
        insert_ping(&conn, ts, r.ok, r.status, r.latency_ms).unwrap_or_else(|e| {
            eprintln!("Failed to insert ping result: {}", e);
        });
    }
}