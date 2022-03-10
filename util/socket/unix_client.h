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

#ifndef BB1957BD_34F0_4EE0_AA8D_38A9F92A33A1
#define BB1957BD_34F0_4EE0_AA8D_38A9F92A33A1

#include "sockaddr_un.h"
#include "unix_socket.h"

namespace FLECS {

class unix_client_t final : public unix_socket_t
{
public:
    explicit unix_client_t(const sockaddr_un_t& addr)
        : unix_socket_t{}
        , _is_connected{}
    {
        if (connect(addr) == 0)
        {
            _is_connected = true;
        }
    }

    explicit unix_client_t(const char* path)
        : unix_client_t{sockaddr_un_t{path}}
    {}

    bool is_connected() const noexcept { return _is_connected; }

private:
    bool _is_connected;
};

} // namespace FLECS

#endif // BB1957BD_34F0_4EE0_AA8D_38A9F92A33A1
