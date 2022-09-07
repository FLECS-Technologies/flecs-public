Here's how you can install Crow on your Mac.
## Getting Crow
### From a [release](https://github.com/CrowCpp/Crow/releases)
#### Archive
Crow provides an archive containing the framework and CMake files, You will only need the `include` folder inside that archive.
#### Single header file
You can also download the `crow_all.h` file which replaces the `include` folder.

### From Source
To get Crow from source, you only need to download the repository (as a `.zip` or through `git clone https://github.com/CrowCpp/Crow.git`).
#### include folder
Once you've downloaded Crow's source code, you only need to take the `include` folder.
#### Single header file
You can generate your own single header file by navigating to the `scripts` folder with your terminal and running the following command:
```
python3 merge_all.py ../include crow_all.h
```
This will generate a `crow_all.h` file which you can use in the following steps
!!! warning

    `crow_all.h` is recommended only for small, possibly single source file projects. For larger projects, it is advised to use the multi-header version.


## Setting up your Crow project
### Using XCode
1. Download and install [Homebrew](https://brew.sh).
2. Run `brew install boost` in your terminal.
3. Create a new XCode project (macOS -> Command Line Tool).
4. Change the following project settings:

    === "Multiple Headers"

        1. Add header search paths for crow's include folder and boost's folder (`/usr/local/include`, `/usr/local/Cellar/boost/include`, and where you placed Crow's `include` folder)
        2. Add linker flags (`-lpthread` and `-lboost_system` if you're running an old version of boost)

    === "Single Header"

        1. Place `crow_all.h` inside your project folder and add it to the project in XCode (you need to use File -> Add files to "project_name")
        2. Add header search paths for boost's folder (`/usr/local/include`, and `/usr/local/Cellar/boost/include`)
        3. Add linker flags (`-lpthread` and `-lboost_system` if you're running an old version of boost)

5. Write your Crow application in `main.cpp` (something like the Hello World example will work).
6. Press `▶` to compile and run your Crow application.


## Building Crow's tests/examples
!!! note

    This tutorial can be used for Crow projects built with CMake as well

1. Download and install [Homebrew](https://brew.sh).
2. Run `brew install cmake boost` in your terminal.
3. Get Crow's source code (the entire source code).
3. Run the following Commands:
    1. `mkdir build`
    2. `cd build`
    3. `cmake ..`
    4. `make -j12`
!!! note

        You can add options like `-DCROW_ENABLE_SSL`, `-DCROW_ENABLE_COMPRESSION`, or `-DCROW_AMALGAMATE` to `cmake ..` to build their tests/examples.

## Compiling using a compiler directly
All you need to do is run the following command:
```
g++ main.cpp -lpthread
```
!!! note

    You'll need to install GCC via `brew install gcc`. the Clang compiler should be part of XCode or XCode command line tools.

You can use arguments like `-DCROW_ENABLE_DEBUG`, `-DCROW_ENABLE_COMPRESSION -lz` for HTTP Compression, or `-DCROW_ENABLE_SSL -lssl` for HTTPS support, or even replace g++ with clang++.
!!! warning

    If you're using a version of boost prior to 1.69, you'll need to add the argument `-lboost_system` in order for you Crow application to compile correctly.
