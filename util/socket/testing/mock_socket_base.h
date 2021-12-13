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

#ifndef FLECS_util_socket_testing_mock_socket_base_h
#define FLECS_util_socket_testing_mock_socket_base_h

#include "gmock/gmock.h"
#include "util/socket/socket.h"

namespace FLECS {

class mock_socket_t : public socket_t
{
public:
    explicit mock_socket_t(int fd)
        : socket_t{fd}
    {}
    mock_socket_t(domain_t domain, type_t type, int protocol)
        : socket_t{domain, type, protocol}
    {}
    mock_socket_t(const mock_socket_t& other) = delete;
    mock_socket_t& operator=(mock_socket_t other) = delete;
    mock_socket_t(mock_socket_t&& other);
    virtual ~mock_socket_t() {}

    MOCK_METHOD(int, accept, (sockaddr * addr, socklen_t* addrlen), (const));
    MOCK_METHOD(int, accept, (sockaddr_in_t & addr), (const));
    MOCK_METHOD(int, bind, (const sockaddr* addr, socklen_t addrlen), (const));
    MOCK_METHOD(int, bind, (const sockaddr_in_t& addr), (const));
    MOCK_METHOD(int, connect, (const sockaddr* addr, socklen_t addrlen), (const));
    MOCK_METHOD(int, connect, (const sockaddr_in_t& addr), (const));
    MOCK_METHOD(int, listen, (int backlog), (const));
    MOCK_METHOD(int, recv, (void* buf, size_t len, int flags), (const));
    MOCK_METHOD(int, send, (const void* buf, size_t len, int flags), (const));

    MOCK_METHOD(bool, is_valid, (), (const, noexcept));
};

} // namespace FLECS

#endif // FLECS_util_socket_testing_mock_socket_base_h
