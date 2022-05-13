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

#include "usage.h"

#include <cstdio>

#include "factory/factory.h"

namespace FLECS {

namespace {
register_module_t<module_usage_t> _reg("usage");
}

module_usage_t::module_usage_t()
{
    using namespace std::placeholders;

    api::register_endpoint("/usage", HTTP_GET, std::bind(&module_usage_t::print_usage, this, _1, _2));
}

http_status_e module_usage_t::print_usage(const nlohmann::json& /*args*/, nlohmann::json& response)
{
    response["usage"] =
        "Usage: flecs [OPTIONS] COMMAND\n\n"
        "Options:\n"
        "    --json         Produce output in JSON format\n"
        "\n"
        "Commands:\n"
        "    app-manager    Manage apps and instances\n"
        "    help           Display help for specific COMMAND\n"
        "    mp             Interact with FLECS marketplace\n"
        "    rpc            Issue RPC for running app\n"
        "    usage          Print this help\n"
        "    version        Print version and exit\n"
        "\n";

    return http_status_e::Ok;
}

} // namespace FLECS
