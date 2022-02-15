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

#include "private/libflecs_private.h"

#include <array>
#include <cstdio>
#include <string>

#include "module_base/errors.h"
#include "util/socket/unix_client.h"

namespace FLECS {
namespace Private {

constexpr const char* FLECS_SOCKET = "/var/run/flecs/flecs.sock";

int libflecs_private_t::run_command(std::string args)
{
    using std::string_literals::operator""s;
    args = "flecs\0"s + args;
    auto client = FLECS::unix_client_t{FLECS_SOCKET};
    if (!client.is_connected())
    {
        std::fprintf(
            stderr,
            "Could not connect to the FLECS socket at %s. Please make sure the FLECS daemon is running.\n",
            FLECS_SOCKET);
        return 1;
    }

    const auto bytes_sent = client.send(args.data(), args.size() + 1, 0);
    if (bytes_sent <= 0)
    {
        std::fprintf(
            stderr,
            "Could not communicate with the FLECS socket at %s: %s (%d)\n",
            FLECS_SOCKET,
            strerror(errno),
            errno);
    }

    auto bytes_received = client.recv(&_return_code, sizeof(_return_code), 0);
    while (bytes_received > 0)
    {
        auto tmp = std::array<char, 4096>{};
        bytes_received = client.recv(tmp.data(), tmp.size(), 0);
        _response.insert(_response.end(), tmp.begin(), tmp.begin() + bytes_received);
    }

    return _return_code;
}

} // namespace Private
} // namespace FLECS
