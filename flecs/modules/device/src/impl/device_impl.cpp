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

#include "flecs/modules/device/impl/device_impl.h"

#include <boost/lexical_cast.hpp>
#include <boost/uuid/random_generator.hpp>
#include <boost/uuid/uuid_io.hpp>
#include <ctime>
#include <fstream>
#include <string>

#ifdef FLECS_MOCK_MODULES
#include "flecs/modules/console/__mocks__/console.h"
#else
#include "flecs/modules/console/console.h"
#endif // FLECS_MOCK_MODULES
#include "flecs/modules/factory/factory.h"
#include "flecs/util/string/string_utils.h"

namespace flecs {
namespace module {
namespace impl {

device_t::device_t(flecs::module::device_t* parent)
    : _parent{parent}
    , _session_id{}
    , _license{}
    , _license_kind{get_license_kind_from_env()}
{}

auto device_t::do_init() //
    -> void
{}

auto device_t::do_deinit() //
    -> void
{}

auto device_t::do_load(const fs::path& base_path) //
    -> result_t
{
    auto result = result_t{0, {}};

    auto [sid_res, sid_msg] = load_session_id(base_path);
    if (sid_res != 0) {
        std::get<0>(result) = -1;
        std::get<1>(result) += sid_msg + "\n";
    }

    auto [lic_res, lic_msg] = load_license(base_path);
    if (lic_res != 0) {
        std::get<0>(result) = -1;
        std::get<1>(result) += lic_msg;
    }

    return result;
}

auto device_t::do_save_session_id(console::session_id_t session_id, const fs::path& base_path) -> result_t
{
    // New session id is only saved if none is present or if it is different and newer
    if (!_session_id.has_value() || (session_id.id() != _session_id.value().id() &&
                                     session_id.timestamp() >= _session_id.value().timestamp())) {
        _session_id = session_id;
        return save_session_id(base_path);
    }
    return {0, {}};
}

auto device_t::do_save(const fs::path& base_path) const //
    -> result_t
{
    auto result = result_t{0, {}};

    auto [sid_res, sid_msg] = save_session_id(base_path);
    if (sid_res != 0) {
        std::get<0>(result) = -1;
        std::get<1>(result) += sid_msg + "\n";
    }

    auto [lic_res, lic_msg] = save_license(base_path);
    if (lic_res != 0) {
        std::get<0>(result) = -1;
        std::get<1>(result) += lic_msg + "\n";
    }

    return result;
}

auto device_t::do_session_id() //
    -> const std::optional<console::session_id_t>&
{
    return _session_id;
}

auto device_t::do_activate_license() //
    -> result_t
{
    auto console_api = std::dynamic_pointer_cast<flecs::module::console_t>(api::query_module("console"));
    auto session_id = _parent->session_id();
    auto result = flecs::module::console_t::license_activation_result_t{};
    if (!_license.has_value()) {
        switch (_license_kind) {
            default:
            case Default:
            case Key:
                result = console_api->activate_license_key();
                break;
            case Serial:
                return {-1, "Licensing via serial number is configured, but no serial number was found"};
        }
    } else {
        result = console_api->activate_license(_license.value(), session_id);
    }
    if (result.error_message.has_value()) {
        return {-1, result.error_message.value()};
    } else if (result.result.has_value()) {
        _license = result.result.value().license_key();
        _session_id = result.result.value().session_id();
        _parent->save();
        return {0, {}};
    }
    return {-1, "Unknown error while activating license"};
}

auto device_t::do_validate_license() //
    -> result_t
{
    auto console_api = std::dynamic_pointer_cast<flecs::module::console_t>(api::query_module("console"));
    auto session_id = _parent->session_id();
    if (!session_id.has_value()) {
        return {0,  {}};
    }
    return console_api->validate_license(session_id.value().id());
}

auto device_t::do_activate_license_for_client() //
    -> crow::response
{
    auto [result, message] = do_activate_license();
    auto response = json_t{};

    if (result == 0) {
        response["additionalInfo"] = "OK";
        return crow::response{crow::status::OK, response.dump()};
    }

    response["additionalInfo"] = message;
    return crow::response{crow::status::INTERNAL_SERVER_ERROR, response.dump()};
}

auto device_t::do_validate_license_for_client() //
    -> crow::response
{
    auto [result, message] = do_validate_license();
    auto response = json_t{};

    switch (result) {
        case 1:
            response["isValid"] = true;
            return crow::response{crow::status::OK, response.dump()};
        case 0:
            response["isValid"] = false;
            return crow::response{crow::status::OK, response.dump()};
        default:
            response["additionalInfo"] = message;
            return crow::response{crow::status::INTERNAL_SERVER_ERROR, response.dump()};
    }
}

auto device_t::load_license(const fs::path& base_path) //
    -> result_t
{
    auto [res, msg] = load_license_file(base_path);
    if (res != 0) {
        switch (_license_kind) {
            case Serial:
                // TODO: Implement reading serial number from system
            case Default:
            default:
            case Key:
                break;
        }
    }
    return {0, {}};
}

auto device_t::load_license_file(const fs::path& base_path) //
    -> result_t
{
    const auto sid_path = base_path / "device" / ".license";
    auto sid_file = std::ifstream{sid_path};
    if (!sid_file.good()) {
        _license = {};
        return {-1, "Could not open .license"};
    }

    std::string license;

    if (!std::getline(sid_file, license)) {
        _license = {};
        return {-1, "Could not read license"};
    }

    _license = license;
    return {0, {}};
}

auto device_t::save_license(const fs::path& base_path) const //
    -> result_t
{
    if (!_license.has_value()) {
        return {0, {}};
    }
    const auto dir = base_path / "device";
    auto ec = std::error_code{};
    fs::create_directories(dir, ec);
    if (ec) {
        return {-1, "Could not create directory"};
    }

    const auto license_path = dir / ".license";
    auto license_file = std::ofstream{license_path, std::ios::out | std::ios::trunc};
    if (!license_file.good()) {
        return {-1, "Could not open .license for writing"};
    }

    license_file << _license.value();

    return {0, {}};
}

auto device_t::load_session_id(const fs::path& base_path) //
    -> result_t
{
    const auto sid_path = base_path / "device" / ".session_id";
    auto sid_file = std::ifstream{sid_path};
    if (!sid_file.good()) {
        _session_id = {};
        return {-1, "Could not open .session_id"};
    }

    std::string id;
    std::time_t timestamp;
    std::string timestamp_line;

    if (!std::getline(sid_file, id) || !std::getline(sid_file, timestamp_line)) {
        _session_id = {};
        return {-1, "Could not read session_id and timestamp"};
    }

    trim(id);
    try {
        boost::lexical_cast<boost::uuids::uuid>(id);
    } catch (boost::exception const& e) {
        _session_id = {};
        return {-1, "Could not parse session_id"};
    }

    try {
        timestamp = std::stoll(timestamp_line);
    } catch (std::exception const& e) {
        return {-1, "Could not parse timestamp"};
    }

    _session_id = console::session_id_t{id, timestamp};

    return {0, {}};
}

auto device_t::save_session_id(const fs::path& base_path) const //
    -> result_t
{
    if (!_session_id.has_value()) {
        return {0, {}};
    }
    const auto dir = base_path / "device";
    auto ec = std::error_code{};
    fs::create_directories(dir, ec);
    if (ec) {
        return {-1, "Could not create directory"};
    }

    const auto sid_path = dir / ".session_id";
    auto sid_file = std::ofstream{sid_path, std::ios::out | std::ios::trunc};
    if (!sid_file.good()) {
        return {-1, "Could not open .session_id for writing"};
    }

    sid_file << _session_id.value().id() << std::endl << _session_id.value().timestamp();

    return {0, {}};
}

auto device_t::get_license_kind_from_env() //
    -> LicenseKind
{
    static const std::unordered_map<std::string, LicenseKind> kind_map = {
        {"LicenseKey", LicenseKind::Key},
        {"Serial", LicenseKind::Serial},
    };
    const char* path = std::getenv("FLECS_LICENSE_KIND");

    if (path != nullptr) {
        auto kind_str = std::string(path);
        auto kind = kind_map.find(kind_str);
        if (kind != kind_map.end()) {
            return kind->second;
        }
    }
    return LicenseKind::Default;
}

} // namespace impl
} // namespace module
} // namespace flecs
