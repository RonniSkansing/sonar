FROM rust:1.41 as builder
WORKDIR /usr/src/sonar
COPY ./ .
RUN cargo install --debug --path .

FROM debian:buster-slim
RUN apt-get update -y && apt-get install openssl -y  && apt-get install ca-certificates
WORKDIR "/opt/sonar/local-mount" 
COPY --from=builder /usr/local/cargo/bin/sonar /usr/local/bin/sonar
RUN mkdir -p /opt/sonar/dashboards/ && mkdir -p /opt/sonar/local-mount

CMD ["bash", "/opt/sonar/local-mount/start.sh"]
