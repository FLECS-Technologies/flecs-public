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

#include "daemon/daemon.h"

#include <getopt.h>

#include <thread>

#include "api/api.h"
#include "signal_handler.h"

namespace FLECS {

daemon_t::daemon_t()
    : _api{}
    , _api_thread{}
{}

int daemon_t::detach()
{
    _api_thread = std::thread{&flecs_api_t::run, &_api};
    pthread_setname_np(_api_thread.native_handle(), "api_thread");

    _api_thread.detach();

    return 0;
}

} // namespace FLECS

constexpr struct option options[] = {{"json", no_argument, nullptr, 0}, {nullptr, no_argument, nullptr, 0}};

int main(int /*argc*/, char** /*argv*/)
{
    FLECS::signal_handler_init();

    auto daemon = FLECS::daemon_t{};
    daemon.detach();

    while (!FLECS::g_stop)
    {
        std::this_thread::sleep_for(std::chrono::milliseconds(1000));
    }
}
