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

#ifndef FLECS_util_socket_tcp_server_h
#define FLECS_util_socket_tcp_server_h

#include "tcp_socket.h"
#include "sockaddr_in.h"

namespace FLECS {

class tcp_server_t final : public tcp_socket_t
{
public:
    tcp_server_t(const sockaddr_in_t& addr, int backlog)
        : tcp_socket_t {}
        , _is_running {}
    {
        if (bind(addr) == 0 && listen(backlog) == 0)
        {
            _is_running = true;
        }
    }

    tcp_server_t(in_port_t in_port, in_addr_t in_addr, int backlog)
        : tcp_server_t { {in_port, in_addr}, backlog }
    {}

    virtual ~tcp_server_t() {}

    bool is_running() const noexcept
         { return _is_running; }

private:
    bool _is_running;
};

} // namespace FLECS

#endif // FLECS_util_socket_tcp_server_h
