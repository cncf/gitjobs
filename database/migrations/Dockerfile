# Build tern
FROM golang:1.24.2-alpine3.21 AS tern
RUN apk --no-cache add git
RUN go install github.com/jackc/tern@latest

# Build final image
FROM alpine:3.21.3
RUN addgroup -S gitjobs && adduser -S gitjobs -G gitjobs
USER gitjobs
WORKDIR /home/gitjobs
COPY --from=tern /go/bin/tern /usr/local/bin
COPY database/migrations .
