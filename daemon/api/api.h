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

#ifndef D409ED4F_76EC_4E01_B2EB_9DCBCF588B5E
#define D409ED4F_76EC_4E01_B2EB_9DCBCF588B5E

#include <crow.h>

#define FLECS_ROUTE(url) CROW_ROUTE(FLECS::flecs_api_t::instance().app(), url)

namespace FLECS {

/*! API for communication with the outside world. Runs an HTTP server handling requests on registered endpoints.
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

private:
    flecs_api_t();
    ~flecs_api_t();

    crow::SimpleApp _app;
};

} // namespace FLECS

#endif // D409ED4F_76EC_4E01_B2EB_9DCBCF588B5E
