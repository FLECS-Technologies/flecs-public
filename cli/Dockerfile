FROM busybox:stable

ARG MACHINE

COPY out/${MACHINE}/flecs /bin/flecs
RUN chmod a+x /bin/flecs

ENTRYPOINT [ "/bin/cp", "-pf", "/bin/flecs", "/host/usr/bin/flecs"  ]
