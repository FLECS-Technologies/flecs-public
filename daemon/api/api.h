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

#pragma once

#include <crow.h>

#define FLECS_ROUTE(url) CROW_ROUTE(FLECS::flecs_api_t::instance().app(), url)
#define FLECS_V2_ROUTE(url) CROW_BP_ROUTE(FLECS::flecs_api_t::instance().v2_api(), url)

namespace FLECS {

/*! API for communication with the outside world. Runs an HTTP server handling requests on
 * registered endpoints.
 */
class flecs_api_t
{
public:
    static auto instance() noexcept //
        -> flecs_api_t&;

    auto app() noexcept //
        -> crow::SimpleApp&
    {
        return _app;
    }

    auto app() const noexcept //
        -> const crow::SimpleApp&
    {
        return _app;
    }

    auto v2_api() noexcept //
        -> crow::Blueprint&
    {
        return _bp_v2;
    }

    auto v2_api() const noexcept //
        -> const crow::Blueprint&
    {
        return _bp_v2;
    }

private:
    flecs_api_t();
    ~flecs_api_t();

    crow::SimpleApp _app;
    crow::Blueprint _bp_v2;
};

} // namespace FLECS
