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

#ifndef A2E0C7BD_9DAC_46A8_9700_D2CE1CE0171E
#define A2E0C7BD_9DAC_46A8_9700_D2CE1CE0171E

#include <sys/socket.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <unistd.h>

#include <algorithm>
#include <cstdint>

#include "util/socket/sockaddr_in.h"
#include "util/socket/sockaddr_un.h"

namespace FLECS {

enum class domain_t : std::int32_t
{
    UNIX = AF_UNIX,
    LOCAL = AF_LOCAL,
    INET = AF_INET,
    INET6 = AF_INET6,
};

enum class type_t : std::int32_t
{
    STREAM = SOCK_STREAM,
    DGRAM = SOCK_DGRAM,
    RAW = SOCK_RAW,
};

inline bool fd_is_socket(int fd)
{
    struct stat stat
    {
    };
    ::fstat(fd, &stat);
    return S_ISSOCK(stat.st_mode);
}

class socket_t
{
public:
    int accept(sockaddr* addr, socklen_t* addrlen) const;
    int accept(sockaddr_in_t& addr) const;
    int accept(sockaddr_un_t& addr) const;
    int bind(const sockaddr* addr, socklen_t addrlen) const;
    int bind(const sockaddr_in_t& addr) const;
    int bind(const sockaddr_un_t& addr) const;
    int connect(const sockaddr* addr, socklen_t addrlen) const;
    int connect(const sockaddr_in_t& addr) const;
    int connect(const sockaddr_un_t& addr) const;
    int listen(int backlog) const;
    int recv(void* buf, size_t len, int flags) const;
    int send(const void* buf, size_t len, int flags) const;

    bool is_valid() const noexcept;

protected:
    explicit socket_t(int fd);
    socket_t(domain_t domain, type_t type, int protocol);
    socket_t(const socket_t& other) = delete;
    socket_t& operator=(socket_t other) = delete;
    socket_t(socket_t&& other);
    virtual ~socket_t();

    friend void swap(socket_t& lhs, socket_t& rhs);

private:
    int _fd;
};

inline int socket_t::accept(sockaddr* addr, socklen_t* addrlen) const
{
    return ::accept(_fd, addr, addrlen);
}

inline int socket_t::accept(sockaddr_in_t& addr) const
{
    addr.size(sizeof(addr._addr));
    return ::accept(_fd, static_cast<sockaddr*>(addr), &addr._size);
}

inline int socket_t::accept(sockaddr_un_t& addr) const
{
    addr.size(sizeof(addr._addr));
    return ::accept(_fd, static_cast<sockaddr*>(addr), &addr._size);
}

inline int socket_t::bind(const sockaddr* addr, socklen_t addrlen) const
{
    return ::bind(_fd, addr, addrlen);
}

inline int socket_t::bind(const sockaddr_in_t& addr) const
{
    return ::bind(_fd, static_cast<const sockaddr*>(addr), addr.size());
}

inline int socket_t::bind(const sockaddr_un_t& addr) const
{
    return ::bind(_fd, static_cast<const sockaddr*>(addr), addr.size());
}

inline int socket_t::connect(const sockaddr* addr, socklen_t addrlen) const
{
    return ::connect(_fd, addr, addrlen);
}

inline int socket_t::connect(const sockaddr_in_t& addr) const
{
    return ::connect(_fd, static_cast<const sockaddr*>(addr), addr.size());
}

inline int socket_t::connect(const sockaddr_un_t& addr) const
{
    return ::connect(_fd, static_cast<const sockaddr*>(addr), addr.size());
}

inline int socket_t::listen(int backlog) const
{
    return ::listen(_fd, backlog);
}

inline int socket_t::recv(void* buf, size_t len, int flags) const
{
    ssize_t size;
    size = ::recv(_fd, buf, len, flags);
    return size;
}

inline int socket_t::send(const void* buf, size_t len, int flags) const
{
    return ::send(_fd, buf, len, flags);
}

inline bool socket_t::is_valid() const noexcept
{
    return _fd != -1;
}

inline socket_t::socket_t(int fd)
    : _fd{fd_is_socket(fd) ? fd : -1}
{}

inline socket_t::socket_t(domain_t domain, type_t type, int protocol)
    : _fd{::socket(static_cast<int>(domain), static_cast<int>(type), protocol)}
{
    const int val = 1;
    setsockopt(_fd, SOL_SOCKET, SO_REUSEPORT, &val, sizeof(val));
}

inline socket_t::socket_t(socket_t&& other)
    : _fd{-1}
{
    swap(*this, other);
}

inline socket_t::~socket_t()
{
    if (is_valid())
    {
        close(_fd);
    }
}

inline void swap(socket_t& lhs, socket_t& rhs)
{
    using std::swap;
    swap(lhs._fd, rhs._fd);
}

} // namespace FLECS

#endif // A2E0C7BD_9DAC_46A8_9700_D2CE1CE0171E
