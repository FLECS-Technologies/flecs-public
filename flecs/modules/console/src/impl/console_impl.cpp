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

#include "flecs/modules/console/impl/console_impl.h"

#include <cpr/cpr.h>

#include "flecs/modules/console/types.h"
#include "flecs/modules/device/device.h"
#include "flecs/modules/factory/factory.h"
#include "flecs/util/json/json.h"

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

auto console_t::do_activate_license(
    const std::optional<std::string>& license, const std::optional<console::session_id_t>& session_id) //
    -> flecs::module::console_t::license_activation_result_t
{
    const auto url = std::string{_parent->base_url()} + "/api/v2/device/license/activate";
    auto res = cpr::Response{};
    // Activation via existing license or serial number
    if (license.has_value()) {
        auto json = json_t{};
        json["licenseKey"] = license.value();
        auto header = session_id.has_value() ? cpr::Header{
                                                   {"X-Session-Id", session_id.value().id()},
                                               } : cpr::Header{};
        res = cpr::Post(cpr::Url(url), cpr::Body(json.dump()), header);
    } else {
        // Activation via license of user
        res = cpr::Post(
            cpr::Url(url),
            cpr::Header{
                {"Authorization", std::string{"Bearer " + _auth.jwt().token()}},
            });
    }

    if (res.status_code == 200) {
        auto response = console::activate_response_t{};
        try {
            parse_json(res.text).get_to(response);
        } catch (...) {
            return {"Invalid JSON response for status code 200", {}};
        }
        return {{}, {response}};
    }

    if (res.status_code == 204) {
        if (!license.has_value()) {
            return {"No license present but console responded with 'already active'", {}};
        }
        auto returned_session_id = console::session_id_t::read_from_header(res.header);
        if (!returned_session_id.has_value()) {
            return {"Console responded with 'already active', but sent no (valid) session id", {}};
        }
        auto response = console::activate_response_data_t(returned_session_id.value(), license.value());
        return {{}, {response}};
    }

    auto response = console::error_response_t{};
    try {
        parse_json(res.text).get_to(response);
    } catch (...) {
        return {"Activation failed with status code " + std::to_string(res.status_code), {}};
    }

    return {response.reason(), {}};
}

auto console_t::do_activate_license_key() //
    -> flecs::module::console_t::license_activation_result_t
{
    return do_activate_license({}, {});
}

/**
 * \return 0 if license is invalid, 1 if license is valid and -1 including an error message if error occurred
 */
auto console_t::do_validate_license(std::string_view session_id) //
    -> result_t
{
    const auto url = std::string{_parent->base_url()} + "/api/v2/device/license/validate";

    const auto res = cpr::Post(
        cpr::Url(std::move(url)),
        cpr::Header{
            {"X-Session-Id", std::string{session_id}},
        });

    if (res.status_code == 200) {
        auto response = console::validate_response_t{};
        try {
            parse_json(res.text).get_to(response);
        } catch (...) {
            return {-1, "Invalid JSON response for status code 200"};
        }
        save_session_id_from_header(res.header);
        if (response.is_valid()) {
            return {1, {}};
        }
        return {0, {}};
    }

    auto response = console::error_response_t{};
    try {
        parse_json(res.text).get_to(response);
    } catch (...) {
        return {-1, "Validation failed with status code " + std::to_string(res.status_code)};
    }

    return {-1, response.reason()};
}

auto console_t::do_acquire_download_token(
    std::string app,
    std::string version,
    std::string session_id) //
    -> std::optional<console::download_token_t>
{
    const auto url = std::string{_parent->base_url()} + "/api/v2/tokens";
    const auto body = json_t({
        {"app", app},
        {"version", version},
    });

    const auto res = cpr::Post(
        cpr::Url(std::move(url)),
        cpr::Header{
            {"X-Session-Id", std::string(session_id)},
        },
        cpr::Body{body.dump()});

    save_session_id_from_header(res.header);
    if (res.status_code == 200) {
        try {
            return parse_json(res.text) //
                .get<console::create_token_response_t>()
                .token();
        } catch (...) {
        }
    }

    if (res.status_code == 204) {
        return console::download_token_t{};
    }

    return {};
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

auto console_t::save_session_id_from_header(const cpr::Header& header) //
    -> void
{
    if (auto session_id = console::session_id_t::read_from_header(header); session_id.has_value()) {
        auto device_api = std::dynamic_pointer_cast<module::device_t>(api::query_module("device"));
        device_api->save_session_id(session_id.value());
    }
}

} // namespace impl
} // namespace module
} // namespace flecs
