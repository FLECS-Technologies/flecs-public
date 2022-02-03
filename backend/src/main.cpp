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

#include <csignal>
#include <iostream>
#include <thread>

#include "backend/http_request_handler.h"
#include "util/container/map_constexpr.h"
#include "util/llhttp_ext/llhttp_ext.h"
#include "util/socket/tcp_server.h"
#include "util/string/literals.h"

auto g_stop = false;

int http_request_handler_thread(FLECS::tcp_socket_t&& conn_socket)
{
    FLECS::http_request_handler_t handler{std::move(conn_socket)};
    auto err = handler.dispatch();
    auto res = handler.send_response(err);
    if (res <= 0)
    {
        return 1;
    }
    return 0;
}

void signal_handler(int)
{
    g_stop = true;
}

int main()
{
    struct sigaction signal_action
    {
    };
    signal_action.sa_handler = &signal_handler;
    sigaction(SIGTERM, &signal_action, nullptr);
    sigaction(SIGINT, &signal_action, nullptr);

    FLECS::tcp_server_t server{42000, INADDR_ANY, 10};
    if (!server.is_running())
    {
        return 1;
    }

    do
    {
        auto conn_socket = FLECS::tcp_socket_t{server.accept(nullptr, nullptr)};
        if (conn_socket.is_valid())
        {
            std::thread handle_thread{&http_request_handler_thread, std::move(conn_socket)};
            handle_thread.detach();
        }
    } while (!g_stop);
}
