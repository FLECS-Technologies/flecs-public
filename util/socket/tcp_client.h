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

#ifndef FLECS_util_socket_tcp_client_h
#define FLECS_util_socket_tcp_client_h

#include <arpa/inet.h>

#include <cerrno>
#include <cstring>

#include "sockaddr_in.h"
#include "tcp_socket.h"

namespace FLECS {

class tcp_client_t final : public tcp_socket_t
{
public:
    tcp_client_t(const sockaddr_in_t& addr)
        : tcp_socket_t{}
        , _is_connected{}
    {
        if (connect(addr) != 0)
        {
            std::fprintf(
                stderr,
                "Could not connect to %s:%d: %d (%s)\n",
                addr.straddr().c_str(),
                ntohs(addr.port()),
                errno,
                strerror(errno));
            return;
        }
        _is_connected = true;
    }

    tcp_client_t(in_port_t in_port, in_addr_t in_addr)
        : tcp_client_t{{in_port, in_addr}}
    {}

    virtual ~tcp_client_t() {}

    bool is_connected() const noexcept { return _is_connected; }

private:
    bool _is_connected;
};

} // namespace FLECS

#endif // FLECS_util_socket_tcp_server_h
