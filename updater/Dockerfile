FROM busybox:stable

ARG MACHINE
ARG ARCH
ARG VERSION

COPY [ \
    "usr/bin/flecs.deb.tar.gz", "usr/bin/flecs", \
    "usr/bin/flecs-backend.deb.tar.gz", \
    "usr/bin/flecs-updater.deb.tar.gz", \
    "/bin/" ]

ENTRYPOINT [ \
    "/bin/cp", "-f", \
    "/bin/flecs.deb.tar.gz", "/bin/flecs", \
    "/bin/flecs-backend.deb.tar.gz", \
    "/bin/flecs-updater.deb.tar.gz", \
    "-t", "/host/usr/bin/" ]
