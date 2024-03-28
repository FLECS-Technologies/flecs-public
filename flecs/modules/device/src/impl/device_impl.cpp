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

auto device_t::do_save_session_id(console::session_id_t session_id) -> result_t
{
    // New session id is only saved if it is different and newer
    if (session_id.id() != _session_id.id() && session_id.timestamp() >= _session_id.timestamp()) {
        _session_id = session_id;
        return _parent->save();
    }
    return {0, {}};
}

auto device_t::do_save(const fs::path& base_path) const //
    -> result_t
{
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

    sid_file << _session_id.id() << std::endl << _session_id.timestamp();

    return {0, {}};
}

auto device_t::do_session_id() //
    -> const console::session_id_t&
{
    if (_session_id.id().empty()) {
        _session_id = console::session_id_t{
            boost::lexical_cast<std::string>(boost::uuids::random_generator{}()),
            std::time(nullptr)};
        _parent->save();
    }

    return _session_id;
}

auto device_t::do_activate_license() //
    -> result_t
{
    auto console_api = std::dynamic_pointer_cast<flecs::module::console_t>(api::query_module("console"));

    return console_api->activate_license(_parent->session_id().id());
}

auto device_t::do_validate_license() //
    -> result_t
{
    auto console_api = std::dynamic_pointer_cast<flecs::module::console_t>(api::query_module("console"));

    return console_api->validate_license(_parent->session_id().id());
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
} // namespace impl
} // namespace module
} // namespace flecs
