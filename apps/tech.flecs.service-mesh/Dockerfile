FROM debian:buster-slim

ARG MACHINE

ADD assets/zenoh-0.5.0-beta.9-${MACHINE}.tar.gz /

COPY assets/entrypoint.sh /

ENTRYPOINT [ "/entrypoint.sh" ]
