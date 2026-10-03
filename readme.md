# AniList Latency Monitor

A small Rust service that checks the [AniList](https://anilist.co) GraphQL API once a minute, records how long it takes to respond, and shows the history on a web dashboard.

- **Poller:** sends a tiny GraphQL request every 60 seconds and times the full response.
- **Storage:** every check is saved to a local SQLite database, including failures and timeouts.
- **API:** a small REST API built with [axum](https://github.com/tokio-rs/axum).
- **Dashboard:** a single HTML page that graphs the data with [Chart.js](https://www.chartjs.org/).

> **Disclaimer:** Originally made purely with ai but i am touching it up and cleaning up some bad code

## How it works

```
every 60s ──> POST https://graphql.anilist.co ──> measure latency
                                                        │
                                                        ▼
browser ──> GET /latency ──> axum ──> SQLite (pings.db)
```

Each check is stored as one row:

| Column       | Meaning                                                              |
|--------------|----------------------------------------------------------------------|
| `ts`         | Unix timestamp in seconds                                            |
| `ok`         | `1` if the request succeeded with no GraphQL errors, otherwise `0`   |
| `status`     | HTTP status code, or `0` if there was no response (timeout, DNS, etc.) |
| `latency_ms` | Time until the full response body was read, in milliseconds          |

A check counts as failed when the request errors, the status is not 2xx, or the response body contains a GraphQL `errors` field.

## Quick start

Requires a recent stable Rust toolchain. Run these from the `backend` folder:

```
cargo run --release
```

Then open <http://127.0.0.1:3000>. The first check runs as soon as the server starts, and the dashboard refreshes itself every 30 seconds.

The database file `pings.db` is created in the directory you run the program from.

## Run with Docker

```
docker build -t anilist-monitor .

docker run -d --name anilist-monitor \
  -p 127.0.0.1:3000:3000 \
  -v anilist-data:/data \
  --restart unless-stopped \
  anilist-monitor
```

- The database lives in the `anilist-data` volume, so history survives rebuilding or removing the container.
- `-p 127.0.0.1:3000:3000` only exposes the port on your own machine. Use `-p 3000:3000` to make it reachable from other devices on your network.
- The server must bind to `0.0.0.0` inside the container, which is what `main.rs` does.

After changing the code or `index.html`, rebuild the image, then run `docker rm -f anilist-monitor` and the `docker run` command again.

### Using an existing database

Mount a host folder at `/data` and run the container as your own user so SQLite can write to it:

```
mkdir -p ~/anilist-data
cp /path/to/your.db ~/anilist-data/pings.db

docker run -d --name anilist-monitor \
  -p 127.0.0.1:3000:3000 \
  -v ~/anilist-data:/data \
  --user "$(id -u):$(id -g)" \
  --restart unless-stopped \
  anilist-monitor
```

The file must be named `pings.db` and contain a `pings` table with the columns above. Mount the folder rather than the single file, because SQLite creates temporary files next to the database.

## API

| Method | Path              | Description                                                          |
|--------|-------------------|----------------------------------------------------------------------|
| GET    | `/`               | The dashboard                                                        |
| GET    | `/latency`        | Recent checks, newest first. Query parameter `limit` (default `60`, max `43200`) |
| GET    | `/latency/latest` | The most recent check, or `404` if there are none yet                |

Example:

```
curl "http://127.0.0.1:3000/latency?limit=2"
```

```json
[
  { "ts": 1759400460, "ok": true, "status": 200, "latency_ms": 142 },
  { "ts": 1759400400, "ok": true, "status": 200, "latency_ms": 156 }
]
```

Since there is one check per minute, `limit` is also the number of minutes of history: 60 is an hour, 1440 is a day, and 43200 is 30 days.

## Dashboard

- **Status headline:** the current response time, or a warning if the last check failed or no check has arrived in the last 3 minutes (usually a sign the poller stopped).
- **Chart:** response time over the selected range (1 hour, 6 hours, 24 hours, 7 days, 30 days). Failed checks leave a gap in the line and a red marker at the bottom.
- **Averaging on long ranges:** ranges with more than about 1,000 checks are grouped into buckets. The line shows each bucket's average, and the tooltip shows the average, the slowest check, and how many checks failed.
- **Summary stats:** median, 95th percentile, slowest, and success rate, calculated from every raw check in the range.
- **Theme:** follows your system light or dark setting.

The page is `index.html`, embedded into the binary at compile time with `include_str!`, so rebuild after editing it. It loads Chart.js and a web font from CDNs, so the browser needs internet access to render it.

## Project layout

```
backend/
├── Cargo.toml
├── Cargo.lock
├── Dockerfile
├── .dockerignore
├── index.html        dashboard, embedded into the binary
└── src/
    └── main.rs       poller, SQLite storage and axum routes
```

## Configuration

These values are currently constants in the code:

| Setting            | Value                       | Where                    |
|--------------------|-----------------------------|--------------------------|
| Check interval     | 60 seconds                  | `poller` in `main.rs`    |
| Request timeout    | 10 seconds                  | `poller` in `main.rs`    |
| Listen address     | `0.0.0.0:3000`              | `main()` in `main.rs`    |
| Database file      | `pings.db` in the working directory | `main()` in `main.rs` |
| Maximum `limit`    | 43200 rows                  | `list` in `main.rs`      |

## Notes

- `rusqlite` needs its `bundled` feature (`cargo add rusqlite --features bundled`) so SQLite is compiled into the binary. Without it the Docker image fails at startup with a missing `libsqlite3.so.0`.
- The dashboard has no authentication. Keep the port bound to localhost, or put it behind a reverse proxy, if you expose it.
- AniList has its own rate limits. One small request a minute is far below them.