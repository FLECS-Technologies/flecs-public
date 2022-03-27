FROM debian:buster-slim

ARG MACHINE

COPY [ \
    "out/${MACHINE}/docker/mqtt_bridge", \
    "/usr/bin/" ]

ENTRYPOINT [ \
    "/usr/bin/mqtt_bridge" ]
