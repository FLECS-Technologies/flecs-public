// Copyright 2021 FLECS Technologies GmbH
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#ifndef FLECS_util_process_process_h
#define FLECS_util_process_process_h

#include <algorithm>
#include <cstdlib>
#include <list>
#include <string>

#include <unistd.h>

namespace FLECS {

class process_t
{
public:
    process_t();

    ~process_t();

    int spawn(const char* path)
        {
            return do_spawn(path, false);
        }

    template <typename Arg, typename... Args>
    int spawn(const char* file, Arg arg, Args... args)
        {
            _args.emplace_back(arg);
            return spawn(file, args...);
        }

    int spawnp(const char* file)
        {
            return do_spawn(file, true);
        }

    template <typename Arg, typename... Args>
    int spawnp(const char* file, Arg arg, Args... args)
        {
            _args.emplace_back(arg);
            return spawnp(file, args...);
        }

    int wait(bool dump_stdout, bool dump_stderr) noexcept;
    void dump_stdout() const noexcept;
    void dump_stderr() const noexcept;
    int exit_code() const noexcept;

    std::string stdout() const noexcept;
    std::string stderr() const noexcept;

private:
    int do_spawn(const char* exec, bool path);

    std::string output(int fd) const noexcept;

    std::list<std::string> _args;
    char _filename_stdout[25];
    char _filename_stderr[25];
    int _fd_stdout;
    int _fd_stderr;
    pid_t _pid;
    int _status;
};

} // namespace FLECS

#endif // FLECS_util_process_process_h
