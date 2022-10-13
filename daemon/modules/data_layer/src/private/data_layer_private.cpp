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

#include "private/data_layer_private.h"

#include "util/json/json.h"

namespace FLECS {
namespace Private {

module_data_layer_private_t::module_data_layer_private_t()
{}

module_data_layer_private_t::~module_data_layer_private_t()
{}

auto module_data_layer_private_t::do_init() //
    -> void
{
    _client.connect(FLUNDER_HOST, FLUNDER_PORT);
}

auto module_data_layer_private_t::do_deinit() //
    -> void
{
    _client.disconnect();
}

auto module_data_layer_private_t::do_browse(std::string_view path, json_t& response) //
    -> crow::status
{
    response["additionalInfo"] = "";

    const auto res = _client.get(path.empty() ? "**" : path);

    if (std::get<0>(res) != 0)
    {
        response["additionalInfo"] = "Could not get data from client";
        return crow::status::INTERNAL_SERVER_ERROR;
    }

    decltype(auto) vars = std::get<1>(res);

    response["data"] = json_t::array();
    for (decltype(auto) it = vars.cbegin(); it != vars.cend(); ++it)
    {
        auto data = json_t{};
        data["key"] = std::string{it->_key};
        data["value"] = std::string{it->_value};
        data["encoding"] = std::string{it->_encoding};
        data["timestamp"] = std::string{it->_timestamp};
        response["data"].push_back(data);
    }

    return crow::status::OK;
}

} // namespace Private
} // namespace FLECS
