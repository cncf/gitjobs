# Build tern
FROM golang:1.25.6-alpine3.22 AS tern
RUN apk --no-cache add git
RUN go install github.com/jackc/tern@latest

# Build final image
FROM alpine:3.22.2
RUN addgroup -S gitjobs && adduser -S gitjobs -G gitjobs
USER gitjobs
WORKDIR /home/gitjobs
COPY --from=tern /go/bin/tern /usr/local/bin
COPY database/migrations .
