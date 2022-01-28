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

#include <getopt.h>

#include <cstdio>

#include "daemon.h"
#include "signal_handler/signal_handler.h"

constexpr struct option options[] = {{"json", no_argument, nullptr, 0}, {nullptr, no_argument, nullptr, 0}};

int print_usage()
{
    std::printf("Usage: flecs [OPTIONS] COMMAND\n\n");
    std::printf("Options:\n");
    std::printf("    --json         Produce output in JSON format\n");
    std::printf("\n");
    std::printf("Commands:\n");
    std::printf("    app-manager    Manage apps and instances\n");
    std::printf("    help           Display help for specific COMMAND\n");
    std::printf("    rpc            Issue RPC for running app\n");
    std::printf("\n");

    return 1;
}

int main(int /*argc*/, char** /*argv*/)
{
    FLECS::signal_handler_init();

    auto daemon = FLECS::daemon_t{};
    return daemon.run();
}
