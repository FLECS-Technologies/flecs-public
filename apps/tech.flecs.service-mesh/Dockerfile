FROM debian:buster-slim

ARG ARCH

ADD assets/zenoh-0.5.0-beta.9-${ARCH}.tar.gz /

COPY assets/entrypoint.sh /

ENTRYPOINT [ "/entrypoint.sh" ]
