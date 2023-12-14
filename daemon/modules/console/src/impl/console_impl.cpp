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

#include "daemon/modules/console/impl/console_impl.h"

#include <cpr/cpr.h>

#include "daemon/modules/console/types.h"
#include "util/json/json.h"

namespace flecs {
namespace module {
namespace impl {

console_t::console_t(flecs::module::console_t* parent)
    : _parent{parent}
    , _auth{}
{}

console_t::~console_t()
{}

auto console_t::do_init() //
    -> void
{}

auto console_t::do_deinit() //
    -> void
{}

auto console_t::do_authentication() const noexcept //
    -> const console::auth_response_data_t&
{
    return _auth;
}

auto console_t::do_activate_license(std::string session_id) //
    -> result_t
{
    const auto url = std::string{_parent->base_url()} + "/api/v2/device/license/activate";

    const auto res = cpr::Post(
        cpr::Url(std::move(url)),
        cpr::Header{
            {"Authorization", std::string{"Bearer " + _auth.jwt().token()}},
            {"X-Session-Id", session_id},
        });

    if (res.status_code == 200) {
        auto response = console::activate_response_t{};
        try {
            parse_json(res.text).get_to(response);
        } catch (...) {
            return {-1, "Invalid JSON response for status code 200"};
        }
        return {0, response.session_id()};
    }

    if (res.status_code == 204) {
        return {0, session_id};
    }

    auto response = console::error_response_t{};
    try {
        parse_json(res.text).get_to(response);
    } catch (...) {
        return {-1, "Activation failed with status code " + std::to_string(res.status_code)};
    }

    return {-1, response.reason()};
}

auto console_t::do_validate_license(std::string_view session_id) //
    -> result_t
{
    const auto url = std::string{_parent->base_url()} + "/api/v2/device/license/validate";

    const auto res = cpr::Post(
        cpr::Url(std::move(url)),
        cpr::Header{
            {"Authorization", std::string{"Bearer " + _auth.jwt().token()}},
            {"X-Session-Id", std::string{session_id}},
        });

    if (res.status_code == 200) {
        auto response = console::validate_response_t{};
        try {
            parse_json(res.text).get_to(response);
        } catch (...) {
            return {-1, "Invalid JSON response for status code 200"};
        }
        if (response.is_valid()) {
            return {0, {}};
        }
        return {-1, "Device is not activated"};
    }

    auto response = console::error_response_t{};
    try {
        parse_json(res.text).get_to(response);
    } catch (...) {
        return {-1, "Validation failed with status code " + std::to_string(res.status_code)};
    }

    return {-1, response.reason()};
}

auto console_t::do_download_manifest(std::string app, std::string version, std::string session_id) //
    -> std::string
{
    const auto url = std::string{_parent->base_url()} + "/api/v2/manifests/" + app + "/" + version;

    const auto res = cpr::Get(
        cpr::Url(std::move(url)),
        cpr::Header{
            {"Authorization", std::string{"Bearer "} + _auth.jwt().token()},
            {"X-Session-Id", std::string(session_id)},
        });

    if (res.status_code == 200) {
        try {
            return parse_json(res.text).at("data").dump();
        } catch (...) {
        }
    }

    return std::string{};
}

auto console_t::do_store_authentication(console::auth_response_data_t auth) //
    -> crow::response
{
    _auth = std::move(auth);

    return crow::response{crow::NO_CONTENT};
}

auto console_t::do_delete_authentication() //
    -> crow::response
{
    _auth = console::auth_response_data_t{};

    return crow::response{crow::NO_CONTENT};
}

} // namespace impl
} // namespace module
} // namespace flecs
