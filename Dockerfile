FROM rust:1.41 as builder
WORKDIR /usr/src/sonar
COPY ./ .
RUN cargo install --debug --path .

FROM debian:buster-slim
RUN apt update -y && apt install openssl -y
WORKDIR "/opt/sonar/" 
COPY --from=builder /usr/local/cargo/bin/sonar /usr/local/bin/sonar
RUN mkdir -p /opt/sonar/dashboards/
RUN sonar init
CMD ["sonar", "run"]
