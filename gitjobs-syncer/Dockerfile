# Build syncer
FROM rust:1-alpine3.22 AS builder
RUN apk --no-cache add musl-dev perl make
WORKDIR /gitjobs
COPY Cargo.* ./
COPY gitjobs-server/Cargo.* gitjobs-server
COPY gitjobs-syncer gitjobs-syncer
WORKDIR /gitjobs/gitjobs-syncer
RUN cargo build --release

# Final stage
FROM alpine:3.22.1
RUN apk --no-cache add ca-certificates && addgroup -S gitjobs && adduser -S gitjobs -G gitjobs
USER gitjobs
WORKDIR /home/gitjobs
COPY --from=builder /gitjobs/target/release/gitjobs-syncer /usr/local/bin
