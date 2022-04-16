FROM debian:buster-slim

ARG MACHINE

COPY [ \
    "docker/mqtt_bridge", \
    "/usr/bin/" ]

ENTRYPOINT [ \
    "/usr/bin/mqtt_bridge" ]
