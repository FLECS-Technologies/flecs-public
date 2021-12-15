FROM debian:bullseye-slim

ARG MACHINE

COPY out/${MACHINE}/flecs-backend /usr/bin/flecs-backend

ENTRYPOINT [ "/usr/bin/flecs-backend" ]