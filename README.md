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
`ARCH` has to be one of `(amd64|arm64|armhf)`

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
