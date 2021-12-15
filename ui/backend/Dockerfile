FROM busybox:stable

ARG MACHINE

COPY out/${MACHINE}/flecs-backend /bin/flecs-backend

ENTRYPOINT [ "/bin/cp", "-pf", "/bin/flecs-backend", "/host/usr/bin/"  ]