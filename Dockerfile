FROM docker.io/rust:latest AS builder

WORKDIR /zns

RUN cargo install diesel_cli --no-default-features --features postgres
COPY . .
RUN cargo install --locked --path zns-daemon

FROM docker.io/debian:bookworm-slim 

WORKDIR /zns

COPY --from=builder /usr/local/cargo/bin/diesel /usr/local/cargo/bin/zns-daemon /usr/local/bin
COPY zns-daemon/diesel.toml .
COPY zns-daemon/migrations/ migrations/

RUN apt update && apt install libpq5 ca-certificates --yes

CMD diesel migration run && zns-daemon
