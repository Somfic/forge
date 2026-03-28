FROM rust:1-bookworm AS builder

RUN curl -fsSL https://bun.sh/install | bash
ENV PATH="/root/.bun/bin:${PATH}"

WORKDIR /app
COPY . .

RUN bun install --cwd frontend --trust
RUN cargo build --release -p forge_server

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    ffmpeg \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/forge /usr/local/bin/forge
COPY --from=builder /app/frontend/apps/dashboard/build /app/frontend/apps/dashboard/build
COPY --from=builder /app/modules/cinema/frontend/build /app/modules/cinema/frontend/build

WORKDIR /app
ENTRYPOINT ["forge"]
