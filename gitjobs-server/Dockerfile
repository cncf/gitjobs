# Build server
FROM rust:1-alpine3.21 AS builder
RUN apk --no-cache add musl-dev perl make
WORKDIR /gitjobs
COPY Cargo.* ./
COPY gitjobs-server gitjobs-server
WORKDIR /gitjobs/gitjobs-server
RUN cargo build --release

# Final stage
FROM alpine:3.21.2
RUN apk --no-cache add ca-certificates && addgroup -S gitjobs && adduser -S gitjobs -G gitjobs
USER gitjobs
WORKDIR /home/gitjobs
COPY --from=builder /gitjobs/target/release/gitjobs-server /usr/local/bin
