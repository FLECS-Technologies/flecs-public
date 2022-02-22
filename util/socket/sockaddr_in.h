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

#ifndef CE0178AA_88CC_483E_9E53_B0CC10CE02B8
#define CE0178AA_88CC_483E_9E53_B0CC10CE02B8

#include <arpa/inet.h>
#include <netinet/in.h>

#include <string>

namespace FLECS {

class sockaddr_in_t final
{
    friend class socket_t;

public:
    sockaddr_in_t() noexcept;
    sockaddr_in_t(in_port_t port, in_addr_t addr) noexcept;

    void port(in_port_t port) noexcept;
    void addr(in_addr_t addr) noexcept;
    void size(socklen_t size) noexcept;

    in_port_t port() const noexcept;
    in_addr_t addr() const noexcept;
    socklen_t size() const noexcept;

    std::string straddr() const noexcept;

    operator sockaddr*() noexcept;
    operator const sockaddr*() const noexcept;

private:
    sockaddr_in _addr;
    socklen_t _size;
};

inline sockaddr_in_t::sockaddr_in_t() noexcept
    : sockaddr_in_t{0, 0}
{}

inline sockaddr_in_t::sockaddr_in_t(in_port_t port, in_addr_t addr) noexcept
    : _addr{}
{
    _addr.sin_family = AF_INET;
    _addr.sin_port = htons(port);
    _addr.sin_addr.s_addr = addr;
    _size = sizeof(_addr);
}

inline void sockaddr_in_t::port(in_port_t port) noexcept
{
    _addr.sin_port = htons(port);
}

inline void sockaddr_in_t::addr(in_addr_t addr) noexcept
{
    _addr.sin_addr.s_addr = htonl(addr);
}

inline void sockaddr_in_t::size(socklen_t size) noexcept
{
    _size = size;
}

inline in_port_t sockaddr_in_t::port() const noexcept
{
    return _addr.sin_port;
}

inline in_addr_t sockaddr_in_t::addr() const noexcept
{
    return _addr.sin_addr.s_addr;
}

inline socklen_t sockaddr_in_t::size() const noexcept
{
    return _size;
}

inline std::string sockaddr_in_t::straddr() const noexcept
{
    char addr[INET_ADDRSTRLEN] = {};
    inet_ntop(AF_INET, &_addr.sin_addr, addr, INET_ADDRSTRLEN);
    return std::string{addr};
}

inline sockaddr_in_t::operator sockaddr*() noexcept
{
    return reinterpret_cast<sockaddr*>(&_addr);
}

inline sockaddr_in_t::operator const sockaddr*() const noexcept
{
    return reinterpret_cast<const sockaddr*>(&_addr);
}

} // namespace FLECS

#endif // CE0178AA_88CC_483E_9E53_B0CC10CE02B8
