FROM rust:alpine

RUN apk add --no-cache musl-dev
 
WORKDIR /recmd

RUN mkdir /tmp/deleteme \
    && cd /tmp/deleteme \
    && cargo init \
    && cargo add serde \
    && rm -rf /tmp/deleteme

CMD ["/bin/sh"]
