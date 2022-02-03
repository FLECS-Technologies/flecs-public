// Copyright 2021-2022 FLECS Technologies GmbH
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

#ifndef FLECS_util_sockaddr_un_h
#define FLECS_util_sockaddr_un_h

#include <sys/socket.h>
#include <sys/un.h>

#include <cstring>

namespace FLECS {

class sockaddr_un_t final
{
    friend class socket_t;

public:
    sockaddr_un_t() noexcept;
    sockaddr_un_t(const char* path) noexcept;

    void path(const char* path) noexcept;
    void size(socklen_t size) noexcept;

    const char* path() const noexcept;
    socklen_t size() const noexcept;

    operator sockaddr*() noexcept;
    operator const sockaddr*() const noexcept;

private:
    sockaddr_un _addr;
    socklen_t _size;
};

inline sockaddr_un_t::sockaddr_un_t() noexcept
    : sockaddr_un_t{""}
{}

inline sockaddr_un_t::sockaddr_un_t(const char* path) noexcept
    : _addr{}
{
    _addr.sun_family = AF_UNIX;
    this->path(path);
    _size = sizeof(_addr);
}

inline void sockaddr_un_t::path(const char* path) noexcept
{
    std::strncpy(_addr.sun_path, path, sizeof(_addr.sun_path) - 1);
}

inline void sockaddr_un_t::size(socklen_t size) noexcept
{
    _size = size;
}

inline const char* sockaddr_un_t::path() const noexcept
{
    return _addr.sun_path;
}

inline socklen_t sockaddr_un_t::size() const noexcept
{
    return _size;
}

inline sockaddr_un_t::operator sockaddr*() noexcept
{
    return reinterpret_cast<sockaddr*>(&_addr);
}

inline sockaddr_un_t::operator const sockaddr*() const noexcept
{
    return reinterpret_cast<const sockaddr*>(&_addr);
}

} // namespace FLECS

#endif // FLECS_util_sockaddr_un_h
