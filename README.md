# FLECS - The World of Automation
We make the world’s automation solutions available at one place, with one click!  

# How To Build
## Cloning the sources
If you are new to FLECS, make sure to clone recursively:
```bash
git clone --recurse-submodules https://github.com/flecs-technologies/flecs-public
```

If you have cloned the source code earlier, make sure to initialize submodules:
```
git submodule update --init
```

## Requirements
### The recommended way
It is recommended to use our official Docker image for building. To do so, run
```bash
docker run -it --rm --name flecs-build -v $(pwd):$(pwd) -w $(pwd) flecspublic.azurecr.io/flecs-build:v4.1.0-snowhare
```

from the repository's root directory.

If you intend to build Docker images as well (such as our System Apps), make sure to mount yout local Docker socket:
```bash
docker run -it --rm --name flecs-build -v $(pwd):$(pwd) -v /run/docker.sock:/run/docker.sock -w $(pwd) flecspublic.azurecr.io/flecs-build:v4.1.0-snowhare
```

**Note:** It is recommended to use the Docker image tag that corresponds to the Git tag you are building.

### The other way
If you don't want to use our Docker image for whatever reason, here is what you will need:

  * CMake >= 3.25
  * Ninja >= 1.11.1
  * gcc >= 12.2.0
  * g++ >= 12.2.0
  * libudev-dev >= 252

While older versions might work, they are neither tested not officially supported.

**Dependencies**

For a list of up-to-date 3rd party dependencies, refer to [LICENSE-3RDPARTY](https://github.com/FLECS-Technologies/flecs-public/tree/main/LICENSE-3RDPARTY).

Our official Docker image to build FLECS contains prebuilt externals for all architectures. They are located in `/usr/local` and can be extracted using

```bash
CONTAINER_ID=`docker create flecspublic.azurecr.io/flecs-build:latest`
docker cp ${CONTAINER_ID}:/usr/local <target dir>
docker rm ${CONTAINER_ID}
```

Alternatively, your distribution of Linux might provide suitable packages in their respective package repositories, official or unofficial.

### The from-scratch way
To also build all dependencies from source, refer to [flecs-build](https://github.com/FLECS-Technologies/flecs-build/) for more information.

## Building
1. Configuration:
```bash
cmake -G Ninja -B build/${ARCH} -DARCH=${ARCH} -DCMAKE_INSTALL_PREFIX=out/${ARCH}
```
`ARCH` has to be one of `(amd64|arm64)`

2. a. Building the whole project
```bash
cmake --build build/${ARCH}
```

2. b. Building single targets
```bash
cmake --build build/${ARCH} --target <target>
```

3. Installing
```bash
cmake --build build/${ARCH} --target install
```

As FLECS is intended to run as Docker container, install will merely prepare the build artifacts for further packaging. `out/${ARCH}/flecsd/docker` will contain runtime binaries and libraries, while `out/${ARCH}/flecsd/pkg` will contain surrounding scripts and init services.

# flecsd runtime configuration

Some settings can be configured via environment variables and a config file. The config will be loaded once on every
startup. The default location for the config file is `/var/lib/flecs/config.toml` which can be overwritten by setting
the environment variable `FLECS_CORE_CONFIG_PATH`. The following `config.toml` with default values will explain all the
settings.

```toml
# The only required field which defines the version of the config file format. Currently, there is only version 1.
version = 1

# The filter applied to the log printed to standard output. Look at
# https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html#directives for the syntax.
# Default: "debug" for debug builds, "info,tower_http=debug,axum::rejection=debug" for release builds
# Environment variable: RUST_LOG.
tracing_filter = "info,tower_http=debug,axum::rejection=debug"

# The base directory of the persistant files written directly by flecsd. The base directories of modules will be derived
# from this if they are not explicitly set.
# Default: "/var/lib/flecs/". 
# Environment variable: FLECS_CORE_BASE_PATH
base_path = "/var/lib/flecs/"

## The location flecsd listens on
## To open a tcp port use
# [listener.TCP]
## To specify the port use
# Port = 8951
## Environment variable: FLECS_CORE_PORT
## Default: 8951
## To specify the bind address use
# bind_address = "::"
## Environment variable: FLECS_CORE_BIND_ADDRESS
## Default "::" with fallback "0.0.0.0" if Ipv6 is not available
## Full tcp example:
# [listener.TCP]
# port = 8951
# bind_address = "::"

## To listen on a unix socket use
## Socket = "/run/flecs/flecsd.sock"
## Environment variable FLECS_CORE_SOCKET_PATH
## Note that FLECS_CORE_PORT and FLECS_CORE_BIND_ADDRESS take precedence over FLECS_CORE_SOCKET_PATH
## Default: 
[listener.UnixSocket]
socket_path = "/run/flecs/flecsd.sock"

[export]
# The base directory for exports, i.e. where exports are created and stored. If this is not set it will be derived from
# the general flecsd base_path.
# Default: "<flecsd-base_path>/export"
# Environment variable: FLECS_CORE_EXPORT_BASE_PATH
base_path = "/var/lib/flecs/export"

# The timeout in seconds for exporting an app image.
# Default: max signed 64bit integer = 9223372036854775807
# Environment variable: FLECS_CORE_EXPORT_TIMEOUT
timeout = 9223372036854775807

[import]
# The base directory for exports, i.e. where imports are extracted and stored. If this is not set it will be derived 
# from the general flecsd base_path.
# Default: "<flecsd-base_path>/import"
# Environment variable: FLECS_CORE_IMPORT_BASE_PATH
base_path = "/var/lib/flecs/import"

# The timeout in seconds for importing an app image.
# Default: max signed 64bit integer = 9223372036854775807
# Environment variable: FLECS_CORE_IMPORT_TIMEOUT
timeout = 9223372036854775807

[floxy]
# The base directory for floxy, e.g. where instance reverse proxy configs are stored. If this is not set it will be
# derived from the general flecsd base_path.
# Default: "<flecsd-base_path>/floxy"
# Environment variable: FLECS_CORE_FLOXY_BASE_PATH
base_path = "/var/lib/flecs/floxy"

# The main configuration file of floxy which is loaded by nginx.
# Default: "/etc/nginx/floxy.conf"
# Environment variable: FLECS_CORE_FLOXY_CONFIG_PATH
config_path = "/etc/nginx/floxy.conf"

[console]
# The console uri the core connects to.
# Default: "https://console-dev.flecs.tech/" for debug builds, "https://console.flecs.tech/" for release builds
# Environment variable: FLECS_CORE_CONSOLE_URI.
uri = "https://console.flecs.tech/"

[instance]
# The base directory for instances, e.g. where instance configs are stored. If this is not set it will be derived from
# the general flecsd base_path.
# Default: "<flecsd-base_path>/instances"
# Environment variable: FLECS_CORE_INSTANCE_BASE_PATH
base_path = "/var/lib/flecs/instances"

# Settings used to create the default network, this only applies to the creation and won't be applied to an existing
# network.
[network]
# The name of the default network.
# Default: "flecs"
# Environment variable: FLECS_CORE_NETWORK_DEFAULT_NETWORK_NAME
default_network_name = "flecs"

# The subnet of the default network in cidr notation.
# Default: "172.21.0.0/16"
# Environment variable: FLECS_CORE_NETWORK_DEFAULT_CIDR_SUBNET
default_cidr_subnet = "172.21.0.0/16"

# The gateway of the default network.
# Default: "172.21.0.0/16"
# Environment variable: FLECS_CORE_NETWORK_DEFAULT_GATEWAY
default_gateway = "172.21.0.1"

# The network kind of the default network.
# Allowed values: "Internal", "Bridge", "MACVLAN", "IpvlanL2", "IpvlanL3"
# Default: "Bridge"
# Environment variable: FLECS_CORE_NETWORK_DEFAULT_NETWORK_KIND
default_network_kind = "Bridge"

# The parent adapter of the default network.
# Default: Not set
# Environment variable: FLECS_CORE_NETWORK_DEFAULT_PARENT_ADAPTER (Set to an empty string for no parent)
#default_parent_adapter = "flecs-parent"

# The options the default network.
# Default: No options
# Environment variable:
#   Variable name: FLECS_CORE_NETWORK_DEFAULT_OPTIONS f
#   Format: Key-Value assignments with '=' separated by ',', e.g. "option1=123,option2=abc"
#   (Set to an empty string for no options)
[network.default_options]
#option1 = 123
#option2 = abc

[app]
# The base directory for apps, i.e. where app information is stored (not the images). If this is not set it will be
# derived from the general flecsd base_path.
# Default: "<flecsd-base_path>/apps"
# Environment variable: FLECS_CORE_APP_BASE_PATH
base_path = "/var/lib/flecs/apps"

[deployment]
# The base directory for deployments, i.e. information how apps are deployed. If this is not set it will be derived from
# the general flecsd base_path.
# Default: "<flecsd-base_path>/deployments"
# Environment variable: FLECS_CORE_DEPLOYMENT_BASE_PATH
base_path = "/var/lib/flecs/deployments"

[manifest]
# The base directory for apps, i.e. where the app manifests are stored. If this is not set it will be derived from the
# general flecsd base_path.
# Default: "<flecsd-base_path>/manifests"
# Environment variable: FLECS_CORE_MANIFEST_BASE_PATH
base_path = "/var/lib/flecs/manifests"

[secret]
# The base directory for secrets, i.e. where license and session information is stored. If this is not set it will be
# derived from the general flecsd base_path.
# Default: "<flecsd-base_path>/device"
# Environment variable: FLECS_CORE_SECRET_BASE_PATH
base_path = "/var/lib/flecs/device"

[auth]
# The time in seconds the certificate of the issuer is cached for, i.e. the time after which the certificate of the
# issuer will be reaquired from the issuer.
# Default: 300
# Environment variable: FLECS_CORE_ISSUER_CERTIFICATE_CACHE_LIFETIME
issuer_certificate_cache_lifetime = 300
# The path of the casbin policy file used for access control.
# Default: "/usr/local/share/flecs/auth/casbin_policy.csv"
# Environment variable: FLECS_CORE_CASBIN_POLICY_PATH
casbin_policy_path = "/usr/local/share/flecs/auth/casbin_policy.csv"
# The path of the casbin model file used for access control.
# Default: "/usr/local/share/flecs/auth/casbin_model.conf"
# Environment variable: FLECS_CORE_CASBIN_MODEL_PATH
casbin_model_path = "/usr/local/share/flecs/auth/casbin_model.conf"
# The path of the initial auth provider flecsport containing the shipped auth provider.
# Default: "/usr/local/lib/flecs/auth/initial_auth_provider.tar"
# Environment variable: FLECS_CORE_INITIAL_AUTH_PROVIDER_FLECSPORT_PATH
initial_auth_provider_flecsport_path = "/usr/local/lib/flecs/auth/initial_auth_provider.tar"

[system]
# The path of the sbom for flecs-core.
# Default: "/usr/local/lib/flecs/sbom/sbom.spdx.json"
# Environment variable: FLECS_CORE_SBOM_SPDX_PATH
core_sbom_spdx_path = "/usr/local/lib/flecs/sbom/sbom.spdx.json"
```