# Built from the repository root: docker build -f crates/phantom-telemetry-server/Dockerfile .
FROM rust:1-slim AS build
WORKDIR /src
# The root manifest only for the shared crate's workspace settings (edition, lints).
COPY Cargo.toml Cargo.toml
COPY crates/phantom-telemetry crates/phantom-telemetry
COPY crates/phantom-telemetry-server crates/phantom-telemetry-server
RUN cargo build --release --locked --manifest-path crates/phantom-telemetry-server/Cargo.toml

FROM debian:stable-slim
COPY --from=build /src/crates/phantom-telemetry-server/target/release/phantom-telemetry-server /usr/local/bin/
# Railway's proxy sets X-Real-IP to the client's address; it is the header the server read first before
# HORIZON_TELEMETRY_CLIENT_IP_HEADER existed, so this keeps the same address for the rate limits (README "Hosting").
ENV HORIZON_TELEMETRY_DB=/data/telemetry.sqlite3 HORIZON_TELEMETRY_TRUST_PROXY=1 \
    HORIZON_TELEMETRY_CLIENT_IP_HEADER=x-real-ip
CMD ["phantom-telemetry-server"]
