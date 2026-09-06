# Build server
FROM rust:1.96.1-alpine3.24 AS builder
RUN apk --no-cache add musl-dev perl make
RUN wget -O /usr/local/bin/tailwindcss https://github.com/tailwindlabs/tailwindcss/releases/download/v4.2.2/tailwindcss-linux-x64-musl
RUN chmod +x /usr/local/bin/tailwindcss
WORKDIR /gitjobs
COPY Cargo.* ./
COPY docs/about.md docs/about.md
COPY gitjobs-server gitjobs-server
COPY gitjobs-syncer gitjobs-syncer
WORKDIR /gitjobs/gitjobs-server
RUN cargo build --release --locked

# Final stage
FROM alpine:3.24.1
RUN apk --no-cache add ca-certificates && addgroup -S gitjobs && adduser -S gitjobs -G gitjobs
USER gitjobs
WORKDIR /home/gitjobs
COPY --from=builder /gitjobs/target/release/gitjobs-server /usr/local/bin
