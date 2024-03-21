// Copyright 2021-2023 FLECS Technologies GmbH
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

#include "flecs/util/process/process.h"

#include <fcntl.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <unistd.h>

#include <cerrno>
#include <cstdlib>
#include <cstring>
#include <iostream>
#include <memory>

#include "flecs/util/process/posix_spawn.h"

namespace flecs {

process_t::process_t()
    : _args{}
    , _filename_stdout{"/tmp/flecs-stdout-XXXXXX"}
    , _filename_stderr{"/tmp/flecs-stderr-XXXXXX"}
    , _fd_stdout{mkostemp(_filename_stdout, 0)}
    , _fd_stderr{mkostemp(_filename_stderr, 0)}
    , _pid{}
    , _status{}
{}

process_t::process_t(process_t&& other)
    : process_t{}
{
    swap(*this, other);
}

process_t& process_t::operator=(process_t other)
{
    swap(*this, other);
    return *this;
}

process_t::~process_t()
{
    close(_fd_stdout);
    close(_fd_stderr);
    if (strlen(_filename_stdout) > 0) {
        unlink(_filename_stdout);
    }
    if (strlen(_filename_stderr) > 0) {
        unlink(_filename_stderr);
    }
}

int process_t::wait(bool dump_stdout, bool dump_stderr) noexcept
{
    auto res = waitpid(_pid, &_status, 0);
    if (dump_stdout) {
        std::cout << stdout();
    }
    if (dump_stderr) {
        std::cerr << stderr();
    }
    return res;
}

int process_t::exit_code() const noexcept
{
    return WEXITSTATUS(_status);
}

std::string process_t::stdout() const noexcept
{
    return output(_fd_stdout);
}

std::string process_t::stderr() const noexcept
{
    return output(_fd_stderr);
}

std::string process_t::output(int fd) const noexcept
{
    ssize_t len = lseek(fd, 0, SEEK_END);
    lseek(fd, 0, SEEK_SET);

    std::string str(len, '\0');
    read(fd, str.data(), len);
    return str;
}

int process_t::do_spawn(const char* exec, bool path)
{
    flecs_posix_spawn_file_actions_t file_actions;

    int res = posix_spawn_file_actions_adddup2(file_actions.pointer(), _fd_stdout, STDOUT_FILENO);
    if (res < 0) {
        return -1;
    }

    res = posix_spawn_file_actions_adddup2(file_actions.pointer(), _fd_stderr, STDERR_FILENO);
    if (res < 0) {
        return -1;
    }

    flecs_posix_spawnattr_t attr;

    std::unique_ptr<char*[]> argv{new char*[_args.size() + 2]};
    argv[0] = const_cast<char*>(exec);
    auto it = _args.begin();
    std::size_t i = 1;

    while (i <= _args.size()) {
        argv[i] = const_cast<char*>(it->c_str());
        ++i;
        ++it;
    }
    argv[i] = nullptr;

    return path ? posix_spawnp(&_pid, exec, file_actions.pointer(), attr.pointer(), argv.get(), environ)
                : posix_spawn(&_pid, exec, file_actions.pointer(), attr.pointer(), argv.get(), environ);
}

void swap(process_t& lhs, process_t& rhs)
{
    using std::swap;

    swap(lhs._args, rhs._args);
    swap(lhs._filename_stdout, rhs._filename_stdout);
    swap(lhs._filename_stderr, rhs._filename_stderr);
    swap(lhs._fd_stdout, rhs._fd_stdout);
    swap(lhs._fd_stderr, rhs._fd_stderr);
    swap(lhs._pid, rhs._pid);
    swap(lhs._status, rhs._status);
}

} // namespace flecs
