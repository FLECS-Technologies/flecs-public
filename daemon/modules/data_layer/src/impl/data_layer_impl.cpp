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

#include "daemon/modules/data_layer/impl/data_layer_impl.h"

#include "util/json/json.h"

namespace flecs {
namespace module {
namespace impl {

data_layer_t::data_layer_t()
{}

data_layer_t::~data_layer_t()
{}

auto data_layer_t::do_init() //
    -> void
{
    _client.connect(flunder::FLUNDER_HOST, flunder::FLUNDER_PORT);
}

auto data_layer_t::do_deinit() //
    -> void
{
    _client.disconnect();
}

auto data_layer_t::do_browse(std::string_view path) //
    -> crow::response
{
    auto response = json_t{};

    response["additionalInfo"] = "";

    if (!_client.is_connected() && (_client.connect() < 0)) {
        response["additionalInfo"] = "Could not establish connection to Service Mesh";
        return {crow::status::INTERNAL_SERVER_ERROR, response.dump()};
    }

    const auto [res, vars] = _client.get(path.empty() ? "**" : path);

    if (res != 0) {
        response["additionalInfo"] = "Could not get data from client";
        return {crow::status::INTERNAL_SERVER_ERROR, response.dump()};
    }

    response["data"] = json_t::array();
    for (decltype(auto) it = vars.cbegin(); it != vars.cend(); ++it) {
        response["data"].push_back(json_t{
            {"key", std::string{it->topic()}},
            {"value", std::string{it->value()}},
            {"encoding", std::string{it->encoding()}},
            {"timestamp", std::string{it->timestamp()}},
        });
    }

    return {crow::status::OK, response.dump()};
}

} // namespace impl
} // namespace module
} // namespace flecs
