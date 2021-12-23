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

#include <getopt.h>
#include <sys/socket.h>
#include <sys/un.h>
#include <unistd.h>

#include <cerrno>
#include <csignal>
#include <cstring>
#include <iostream>
#include <memory>
#include <string>
#include <thread>
#include <vector>

#include "daemon.h"
#include "service/service_table.h"
#include "util/socket/unix_server.h"
#include "util/string/literals.h"

constexpr struct option options[] = {{"json", no_argument, nullptr, 0}, {nullptr, no_argument, nullptr, 0}};

int print_usage()
{
    printf("Usage: flecs [OPTIONS] COMMAND\n\n");
    printf("Options:\n");
    printf("    --json         Produce output in JSON format\n");
    printf("\n");
    printf("Commands:\n");
    printf("    app-manager    Manage apps and instances\n");
    printf("    help           Display help for specific COMMAND\n");
    printf("    rpc            Issue RPC for running app\n");
    printf("\n");

    return 1;
}

auto g_stop = false;

void signal_handler(int)
{
    g_stop = true;
}

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

int flecs_thread(FLECS::unix_socket_t&& conn_socket)
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

    auto it = FLECS::make_service_table.find(args[0]);
    if (it != FLECS::make_service_table.end())
    {
        err = it->second()->process(args.size() - 1, &args.data()[1]);
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

int main(int /*argc*/, char** /*argv*/)
{
    struct sigaction signal_action
    {
    };
    signal_action.sa_handler = &signal_handler;
    sigaction(SIGTERM, &signal_action, nullptr);
    sigaction(SIGINT, &signal_action, nullptr);

    auto server = FLECS::unix_server_t{FLECS::FLECS_SOCKET, 10};
    if (!server.is_running())
    {
        return 1;
    }

    do
    {
        auto conn_socket = FLECS::unix_socket_t{server.accept(nullptr, nullptr)};
        if (conn_socket.is_valid())
        {
            std::thread handle_thread{&flecs_thread, std::move(conn_socket)};
            handle_thread.detach();
        }
    } while (!g_stop);
}
