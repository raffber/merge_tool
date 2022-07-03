FROM ubuntu:20.04

ENV RUSTUP_HOME=/rust CARGO_HOME=/cargo PATH=/cargo/bin:/rust/bin:$PATH HOME=/home TZ=Europe/Zurich

COPY docker-install.sh /root
RUN /root/docker-install.sh
