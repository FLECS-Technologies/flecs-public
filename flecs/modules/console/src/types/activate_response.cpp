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

#include "flecs/modules/console/types/activate_response.h"

namespace flecs {
namespace console {

activate_response_data_t::activate_response_data_t(console::session_id_t session_id, std::string license_key)
    : _session_id{std::move(session_id)}
    , _license_key{std::move(license_key)}
{}


auto activate_response_data_t::session_id() const noexcept //
    -> const console::session_id_t&
{
    return _session_id;
}

auto activate_response_data_t::license_key() const noexcept //
    -> const std::string&
{
    return _license_key;
}

auto from_json(const json_t& j, activate_response_data_t& response) //
    -> void
{
    j.at("sessionId").get_to(response._session_id);
    j.at("licenseKey").get_to(response._license_key);
}

auto to_json(json_t& j, const activate_response_data_t& response) //
    -> void
{
    j = json_t({
        {"sessionId", response.session_id()},
        {"licenseKey", response.license_key()}
    });
}

auto from_json(const json_t& j, activate_response_t& response) //
    -> void
{
    from_json(j, static_cast<base_response_t&>(response));
    from_json(j.at("data"), static_cast<activate_response_data_t&>(response));
}

auto to_json(json_t& j, const activate_response_t& response) //
    -> void
{
    to_json(j, static_cast<const base_response_t&>(response));
    to_json(j["data"], static_cast<const activate_response_data_t&>(response));
}

} // namespace console
} // namespace flecs
