# Tracing

## Configuration

Tracing in this crate can be controlled via two environment variables. `RUST_BACKTRACE=1` enables backtraces for all
errors. `RUST_BACKTRACE=0` disables them. If this environment variable is not set, backtraces
are controlled by the build type: Debug builds have them enabled while release builds have them turned off. `RUST_LOG`
controls the logging. At startup, a filter is constructed from the value of this variable, controlling which log
messages are displayed. For the syntax, see
the [EnvFilter](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html#example-syntax)
documentation of [tracing-subscriber](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/index.html). If the
variable is not set at startup, a default filter is used depending on the build type.  
Debug: `debug`  
Release: `info,tower_http=debug,axum::rejection=debug`

## Development Guidelines

We use the [tracing](https://docs.rs/tracing/latest/tracing/index.html) crate to collect log information.
Use [Spans](https://docs.rs/tracing/latest/tracing/struct.Span.html) to describe processes that take some time, such as
sending a request, handling a request or processing a job. Attach identifying records like a job id or request url.
Use [Events](https://docs.rs/tracing/latest/tracing/struct.Event.html) for events that occur inside or outside a span,
such as the occurrence of an error.

### Log Levels

| Level | Use case                                                             |
|-------|----------------------------------------------------------------------|
| ERROR | Unexpected errors (e.g. invalid response, 5xx responses)             |
| WARN  | Expected errors (e.g. request for unknown app, 4xx responses)        |
| INFO  | High level important information (e.g. startup, requests, responses) |
| DEBUG | Low level important information (e.g. upgrade of manifest)           |
| TRACE | Everything else                                                      |

