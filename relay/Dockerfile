# Relay for FactoryPlanner online multiplayer
FROM rust:1-bookworm AS build
WORKDIR /app
COPY Cargo.toml Cargo.toml
COPY src src
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=build /app/target/release/factory_relay /usr/local/bin/factory_relay
ENV PORT=7788
EXPOSE 7788
CMD ["factory_relay"]
