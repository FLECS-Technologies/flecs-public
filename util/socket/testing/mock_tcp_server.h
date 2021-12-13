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

#ifndef FLECS_util_socket_mock_tcp_server_h
#define FLECS_util_socket_mock_tcp_server_h

#include "../sockaddr_in.h"
#include "mock_tcp_socket.h"

namespace FLECS {

class mock_tcp_server_t final : public mock_tcp_socket_t
{
public:
    mock_tcp_server_t(const sockaddr_in_t& addr, int backlog)
        : mock_tcp_socket_t{}
    {}

    mock_tcp_server_t(in_port_t in_port, in_addr_t in_addr, int backlog)
        : mock_tcp_socket_t{{in_port, in_addr}, backlog}
    {}

    virtual ~mock_tcp_server_t() {}

    MOCK_METHOD(bool, is_running, ());
};

} // namespace FLECS

#endif // FLECS_util_socket_mock_tcp_server_h
