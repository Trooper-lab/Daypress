# syntax=docker/dockerfile:1

# ---- Build stage -------------------------------------------------------------------------
FROM rust:1-bookworm AS build
WORKDIR /src
COPY . .
# The cache mounts keep the crate registry and build output between builds; the binary is
# copied out because cache mounts are not part of the image.
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/target \
    cargo build --release --locked -p neopress-server --bin neopress \
    && cp target/release/neopress /usr/local/bin/neopress

# ---- Runtime stage -----------------------------------------------------------------------
FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*

# Neopress keeps its state relative to the working directory: the SQLite database
# (DATABASE_PATH), media in uploads/, backup snapshots in backups/ and imported theme files in
# static/themes/. All of it lives in the /data volume.
RUN useradd --system --uid 10001 --user-group --home-dir /data --shell /usr/sbin/nologin neopress \
    && mkdir -p /data/uploads /data/backups /data/static/themes \
    && chown -R neopress:neopress /data

COPY --from=build /usr/local/bin/neopress /usr/local/bin/neopress

USER neopress
WORKDIR /data
ENV PORT=8090 \
    DATABASE_PATH=/data/neopress.sqlite \
    RUST_LOG=neopress_server=info,tower_http=info
VOLUME ["/data"]
EXPOSE 8090

HEALTHCHECK --interval=30s --timeout=5s --start-period=15s --retries=3 \
    CMD curl -fsS "http://127.0.0.1:${PORT}/robots.txt" > /dev/null || exit 1

ENTRYPOINT ["neopress"]
