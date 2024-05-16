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

#include "flecs/modules/device/device.h"

#include "flecs/modules/device/impl/device_impl.h"
#include "flecs/modules/factory/factory.h"

namespace flecs {
namespace module {


device_t::device_t()
    : _impl{new impl::device_t{this}}
{}

device_t::~device_t() = default;

auto device_t::save_session_id(console::session_id_t session_id, const fs::path& base_path) //
    -> result_t
{
    return _impl->do_save_session_id(session_id, base_path);
}

auto device_t::session_id() //
    -> const std::optional<console::session_id_t>&
{
    return _impl->do_session_id();
}

auto device_t::activate_license() //
    -> result_t
{
    return _impl->do_activate_license();
}

auto device_t::validate_license() //
    -> result_t
{
    return _impl->do_validate_license();
}

auto device_t::create_license_info() //
    -> crow::response
{
    return _impl->do_create_license_info();
}

auto device_t::activate_license_for_client() //
    -> crow::response
{
    return _impl->do_activate_license_for_client();
}

auto device_t::validate_license_for_client() //
    -> crow::response
{
    return _impl->do_validate_license_for_client();
}

auto device_t::do_init() //
    -> void
{
    FLECS_V2_ROUTE("/device/license/activation/status").methods("GET"_method)([this]() {
        return validate_license_for_client();
    });
    FLECS_V2_ROUTE("/device/license/info").methods("GET"_method)([this]() {
        return create_license_info();
    });

    FLECS_V2_ROUTE("/device/license/activation").methods("POST"_method)([this]() {
        return activate_license_for_client();
    });
    _impl->do_init();
}

auto device_t::do_deinit() //
    -> void
{
    _impl->do_deinit();
}

auto device_t::do_load(const fs::path& base_path) //
    -> result_t
{
    return _impl->do_load(base_path);
}

auto device_t::do_save(const fs::path& base_path) const //
    -> result_t
{
    return _impl->do_save(base_path);
}

} // namespace module
} // namespace flecs
