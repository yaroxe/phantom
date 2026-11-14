# The Play with friends relay, for anyone to run (docs/play-with-friends.md, "Running your own relay").
# Build context: the repository root (the workspace's manifests and lockfile):
#   docker build -f crates/phantom-relay/Dockerfile -t phantom-relay .
FROM rust:1-slim-bookworm AS build
WORKDIR /src
COPY . .
RUN cargo build --release --locked -p phantom-relay \
    && cp target/release/phantom-relay /phantom-relay

FROM debian:bookworm-slim
COPY --from=build /phantom-relay /usr/local/bin/phantom-relay
ENV PORT=7000
EXPOSE 7000
USER nobody
CMD ["phantom-relay"]
