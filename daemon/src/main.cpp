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

#include "api/api.h"
#include "factory/factory.h"
#include "util/signal_handler/signal_handler.h"

int main(int argc, char* argv[])
{
    const auto bindaddr = argc > 1 ? argv[1] : "127.0.0.1";

    flecs::api::init_modules();
    flecs::flecs_api_t::instance().app().multithreaded().port(8951).bindaddr(bindaddr).run();

    flecs::g_stop = true;

    flecs::api::deinit_modules();

    return 0;
}
