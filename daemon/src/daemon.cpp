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

#include "daemon/include/daemon.h"

#include <atomic>
#include <csignal>
#include <cstdio>
#include <thread>
#include <vector>

#include "service/service_app_manager.h"
#include "service/service_errors.h"
#include "service/service_factory.h"
#include "service/service_help.h"
#include "service/service_rpc.h"
#include "util/string/literals.h"

namespace FLECS {

auto g_stop = std::atomic_bool{};

std::vector<char*> parse_args(char* const& str, size_t len)
{
    auto res = std::vector<char*>{};

    auto last = size_t{};
    for (size_t i = 0; i < len; ++i)
    {
        if (str[i] == '\0')
        {
            res.emplace_back(&str[last]);
            last = i + 1;
        }
    }

    return res;
}

void daemon_t::signal_handler(int)
{
    g_stop = true;
}

daemon_t::daemon_t()
    : _service_table{}
    , _server{FLECS::FLECS_SOCKET, 10}
{
    if (!_server.is_running())
    {
        exit(EXIT_FAILURE);
    }

    struct sigaction signal_action
    {
    };
    signal_action.sa_handler = &daemon_t::signal_handler;
    sigaction(SIGTERM, &signal_action, nullptr);
    sigaction(SIGINT, &signal_action, nullptr);

    _service_table.emplace("app-manager", FLECS::make_service<FLECS::service_app_manager_t>());
    _service_table.emplace("help", FLECS::make_service<FLECS::service_help>());
    _service_table.emplace("rpc", FLECS::make_service<FLECS::service_rpc>());
}

int daemon_t::run()
{
    do
    {
        auto conn_socket = FLECS::unix_socket_t{_server.accept(nullptr, nullptr)};
        if (conn_socket.is_valid())
        {
            std::thread handle_thread{&daemon_t::process, this, std::move(conn_socket)};
            handle_thread.detach();
        }
    } while (!g_stop);
}

int daemon_t::process(FLECS::unix_socket_t&& conn_socket)
{
    char template_stdout[18] = "/tmp/flecs-XXXXXX";
    char template_stderr[18] = "/tmp/flecs-XXXXXX";

    auto fd_stdout = mkstemp(template_stdout);
    auto fd_stderr = mkstemp(template_stderr);

    dup2(fd_stdout, STDOUT_FILENO);
    dup2(fd_stderr, STDERR_FILENO);

    using FLECS::operator""_kiB;
    char buf[128_kiB];

    auto n_bytes = conn_socket.recv(buf, sizeof(buf), 0);
    if (n_bytes <= 0)
    {
        return 1;
    }

    FLECS::service_error_e err = FLECS::FLECS_USAGE;
    auto args = parse_args(buf, n_bytes);
    if (args.size() < 1)
    {
        err = FLECS::FLECS_USAGE;
        conn_socket.send(&err, sizeof(err), 0);
        return err;
    }

    auto it = _service_table.find(args[1]);
    if (it != _service_table.end())
    {
        err = it->second->process(args.size() - 2, &args.data()[2]);
    }

    fflush(stdout);
    fflush(stderr);

    conn_socket.send(&err, sizeof(err), 0);

    auto file_stdout = fopen(template_stdout, "r");
    auto s = fread(buf, 1, sizeof(buf), file_stdout);
    conn_socket.send(buf, s, 0);

    unlink(template_stdout);
    unlink(template_stderr);

    return err;
}

} // namespace FLECS
