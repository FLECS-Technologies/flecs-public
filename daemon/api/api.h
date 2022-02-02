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

#ifndef FLECS_daemon_api_h
#define FLECS_daemon_api_h

#include <map>
#include <memory>

#include "modules/module.h"
#include "util/socket/unix_server.h"
#include "util/string/comparator.h"

namespace FLECS {

constexpr const char* FLECS_SOCKET = "/var/run/flecs/flecs.sock";

class socket_api_t
{
public:
    socket_api_t();

    int run();

private:
    int process(FLECS::unix_socket_t&& conn_socket);

    using service_table_t = std::map<const char*, std::shared_ptr<module_t>, string_comparator_t>;
    service_table_t _service_table;

    FLECS::unix_server_t _server;
};

} // namespace FLECS

#endif // FLECS_daemon_api_h
