FROM debian:buster-slim

ARG ARCH

ADD assets/zenoh-0.6.0-beta.1-${ARCH}.tar.gz /

COPY assets/entrypoint.sh /

ENTRYPOINT [ "/entrypoint.sh" ]
