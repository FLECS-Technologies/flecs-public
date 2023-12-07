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

#include "daemon/modules/device/impl/device_impl.h"

#include <boost/lexical_cast.hpp>
#include <boost/uuid/random_generator.hpp>
#include <boost/uuid/uuid_io.hpp>
#include <fstream>

#include "util/string/string_utils.h"

namespace FLECS {
namespace module {
namespace impl {

device_t::device_t()
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
        _session_id.clear();
        return {-1, "Could not open .session_id"};
    }

    sid_file >> _session_id;
    trim(_session_id);

    try {
        boost::lexical_cast<boost::uuids::uuid>(_session_id);
    } catch (...) {
        _session_id.clear();
        return {-1, "Could not parse session_id"};
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

    sid_file << _session_id;

    return {0, {}};
}

auto device_t::do_session_id() //
    -> const std::string&
{
    if (_session_id.empty()) {
        _session_id = boost::lexical_cast<std::string>(boost::uuids::random_generator{}());
    }

    return _session_id;
}

} // namespace impl
} // namespace module
} // namespace FLECS