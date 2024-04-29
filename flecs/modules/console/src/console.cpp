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

#include "flecs/modules/console/console.h"

#include "flecs/modules/factory/factory.h"

namespace flecs {
namespace module {

namespace {
register_module_t<console_t> _reg("console");
}

console_t::console_t()
    : _rust_impl{new_console(std::string{base_url()})}
{}

console_t::~console_t() = default;

auto console_t::do_init() //
    -> void
{
    FLECS_V2_ROUTE("/console/authentication").methods("PUT"_method)([this](const crow::request& req) {
        auto response = json_t{};
        const auto args = parse_json(req.body);
        REQUIRED_TYPED_JSON(args, auth, console::auth_response_data_t);

        return store_authentication(auth);
    });

    FLECS_V2_ROUTE("/console/authentication")
        .methods("DELETE"_method)([this](const crow::request& /* req */) { return delete_authentication(); });
}

auto console_t::do_deinit() //
    -> void
{}

auto console_t::authentication() const noexcept //
    -> Authentication
{
    return _rust_impl->authentication();
}

auto console_t::activate_license(std::string session_id) //
    -> result_t
{
    try {
        return result_t {0, _rust_impl->activate_license(session_id)};
    } catch (const rust::Error &e) {
        std::cerr << e.what() << "\n";
        return result_t { -1, e.what()};
    }
}

auto console_t::validate_license(std::string_view session_id) //
    -> result_t
{
    try {
        auto valid = _rust_impl->validate_license(std::string{session_id});
        return result_t { valid ? 1 : 0, {}};
    } catch (const rust::Error &e) {
        std::cerr << e.what() << "\n";
        return result_t { -1, e.what()};
    }
}

auto console_t::download_manifest(std::string app, std::string version, std::string session_id) //
    -> std::string
{
    try {
        return std::string(_rust_impl->download_manifest(app, version, session_id));
    } catch (const rust::Error &e) {
        std::cerr << e.what() << "\n";
        return {};
    }
}

auto console_t::acquire_download_token(std::string app, std::string version, std::string session_id) //
    -> std::optional<DownloadToken>
{
    try {
        return _rust_impl->acquire_download_token(app, version, session_id);
    } catch (const rust::Error &e) {
        std::cerr << e.what() << "\n";
        return {};
    }
}

auto console_t::store_authentication(console::auth_response_data_t auth) //
    -> crow::response
{
    auto rust_auth = Authentication{
        User{
            auth.user().id(),
            auth.user().user_email(),
            auth.user().user_login(),
            auth.user().display_name()},
        Jwt{
            auth.jwt().token(),
            auth.jwt().token_expires(),
        },
        FeatureFlags{
            auth.feature_flags().is_vendor(),
            auth.feature_flags().is_white_labeled(),
        }};
    auto return_code = _rust_impl->store_authentication(rust_auth);
    return crow::response{return_code};
}

auto console_t::delete_authentication() //
    -> crow::response
{
    auto return_code = _rust_impl->delete_authentication();
    return crow::response{return_code};
}

} // namespace module
} // namespace flecs
