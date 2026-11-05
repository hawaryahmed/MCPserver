FROM rust:1.99.1-bookworm AS build
WORKDIR /build
COPY server/Cargo.toml server/Cargo.lock ./
COPY server/src ./src
RUN cargo build --release --locked

FROM debian:bookworm-slim
RUN groupadd --gid 10001 mcp && useradd --uid 10001 --gid mcp --no-create-home mcp \
    && mkdir -p /var/data && chown mcp:mcp /var/data
COPY --from=build /build/target/release/enisa-sme-mcp /usr/local/bin/enisa-sme-mcp
ENV HOST=0.0.0.0 PORT=10000 ENISA_MCP_DATA_DIR=/var/data
USER mcp
EXPOSE 10000
CMD ["enisa-sme-mcp"]
