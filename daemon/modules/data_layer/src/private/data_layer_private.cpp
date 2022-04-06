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

#include <json/json.h>

namespace FLECS {
namespace Private {

module_data_layer_private_t::module_data_layer_private_t()
{}

module_data_layer_private_t::~module_data_layer_private_t()
{}

http_status_e module_data_layer_private_t::do_browse(const std::string& path, Json::Value& response)
{
    response["additionalInfo"] = "";

    const auto res = _client.get(path.empty() ? "/**" : path);

    if (std::get<0>(res) != 0)
    {
        response["additionalInfo"] = "Could not get data from client";
        return http_status_e::InternalServerError;
    }

    decltype(auto) vars = std::get<1>(res);

    response["data"] = Json::Value{Json::arrayValue};
    for (decltype(auto) it = vars.cbegin(); it != vars.cend(); ++it)
    {
        auto data = Json::Value{};
        data["key"] = it->_key;
        data["value"] = it->_value;
        data["encoding"] = it->_encoding;
        data["timestamp"] =
            std::string{it->_timestamp, std::find(it->_timestamp, it->_timestamp + std::strlen(it->_timestamp), '/')};
        response["data"].append(data);
    }

    return http_status_e::Ok;
}

} // namespace Private
} // namespace FLECS
