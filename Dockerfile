# syntax=docker/dockerfile:1

# ---- build stage ----
FROM rust:1-bookworm AS build
WORKDIR /app

# Build dependencies first so they stay cached when only your code changes
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs \
    && cargo build --release \
    && rm -rf src

# Now build the real app (index.html is embedded with include_str!)
COPY src ./src
COPY index.html ./index.html
RUN touch src/main.rs && cargo build --release

# ---- runtime stage ----
FROM debian:bookworm-slim

# ca-certificates + libssl for HTTPS requests to AniList
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --create-home --home-dir /data app

COPY --from=build /app/target/release/backend /usr/local/bin/backend

# The app opens pings.db relative to its working directory, so /data holds the database
USER app
WORKDIR /data
VOLUME /data
EXPOSE 3000

CMD ["backend"]