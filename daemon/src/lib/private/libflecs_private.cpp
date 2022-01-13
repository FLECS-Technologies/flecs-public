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

#include "lib/private/libflecs_private.h"

#include <array>
#include <cstdio>
#include <string>

#include "daemon.h"
#include "service/service_errors.h"
#include "util/socket/unix_client.h"

namespace FLECS {
namespace Private {

int run_flecs_command_private(const std::string& args)
{
    auto client = FLECS::unix_client_t{FLECS_SOCKET};
    if (!client.is_connected())
    {
        std::fprintf(
            stderr,
            "Could not connect to the FLECS socket at %s. Please make sure the FLECS daemon is running.\n",
            FLECS_SOCKET);
        return 1;
    }

    const auto bytes_sent = client.send(args.data(), args.size(), 0);
    if (bytes_sent <= 0)
    {
        std::fprintf(
            stderr,
            "Could not communicate with the FLECS socket at %s: %s (%d)\n",
            FLECS::FLECS_SOCKET,
            strerror(errno),
            errno);
    }

    auto res = FLECS::service_error_e{};
    auto rdbuf = std::string{};
    auto bytes_received = client.recv(&res, sizeof(res), 0);
    while (bytes_received > 0)
    {
        auto tmp = std::array<char, 4096>{};
        bytes_received = client.recv(tmp.data(), tmp.size(), 0);
        rdbuf.insert(rdbuf.end(), tmp.begin(), tmp.begin() + bytes_received);
    }

    std::fprintf(stdout, "%s", rdbuf.c_str());

    return res;
}

} // namespace Private
} // namespace FLECS