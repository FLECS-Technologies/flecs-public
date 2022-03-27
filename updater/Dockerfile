FROM busybox:stable

ARG MACHINE
ARG ARCH
ARG VERSION

COPY [ \
    "out/${MACHINE}/usr/bin/flecs.deb.tar.gz", \
    "out/${MACHINE}/usr/bin/flecs-updater.deb.tar.gz", \
    "/bin/" ]

ENTRYPOINT [ \
    "/bin/cp", "-f", \
    "/bin/flecs.deb.tar.gz", \
    "/bin/flecs-updater.deb.tar.gz", \
    "-t", "/host/usr/bin/" ]
