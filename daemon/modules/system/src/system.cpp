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

#include "system.h"

#include <cstdio>

#include "factory/factory.h"

namespace FLECS {

namespace {
register_module_t<module_system_t> _reg("system");
}

module_system_t::module_system_t()
{
    using namespace std::placeholders;

    api::register_endpoint("/system/ping", HTTP_GET, std::bind(&module_system_t::ping, this, _1, _2));
}

http_status_e module_system_t::ping(const json_t & /*args*/, json_t &response)
{
    response["additionalInfo"] = "OK";
    return http_status_e::Ok;
}

} // namespace FLECS
