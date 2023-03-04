FROM rust:alpine

RUN apk add --no-cache musl-dev
 
WORKDIR /recmd

RUN mkdir /tmp/deleteme \
    && cd /tmp/deleteme \
    && cargo init \
    && cargo add serde \
    && rm -rf /tmp/deleteme

RUN rustup toolchain install nightly-x86_64-unknown-linux-musl
RUN rustup component add rust-src --toolchain nightly-x86_64-unknown-linux-musl

CMD ["/bin/sh"]
