// Copyright 2021-2023 FLECS Technologies GmbH
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

#include "flecs/util/signal_handler/signal_handler.h"

#include <atomic>
#include <csignal>

namespace flecs {

std::atomic_bool g_stop{};

void signal_handler(int)
{
    g_stop = true;
}

void signal_handler_init()
{
    g_stop = false;

    struct sigaction signal_action
    {
    };
    signal_action.sa_handler = &flecs::signal_handler;
    sigaction(SIGTERM, &signal_action, nullptr);
    sigaction(SIGINT, &signal_action, nullptr);
}

} // namespace flecs
