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
namespace impl {

module_device_t::module_device_t()
{}

auto module_device_t::do_init() //
    -> void
{}

auto module_device_t::do_deinit() //
    -> void
{}

auto module_device_t::do_load(const fs::path& base_path) //
    -> void
{
    auto sid_file = std::ifstream{base_path / "device" / ".session_id"};

    if (!sid_file.good()) {
        _session_id = {};
        return;
    }

    sid_file >> _session_id;
    trim(_session_id);

    try {
        boost::lexical_cast<boost::uuids::uuid>(_session_id);
    } catch (...) {
        _session_id = {};
    }
}

auto module_device_t::do_save(const fs::path& base_path) const //
    -> void
{
    const auto path = base_path / "device";
    auto ec = std::error_code{};
    fs::create_directories(path, ec);
    if (ec) {
        return;
    }

    auto sid_file = std::ofstream{path / ".session_id"};
    if (!sid_file.good()) {
        return;
    }

    sid_file << _session_id;
}

auto module_device_t::do_session_id() //
    -> const std::string&
{
    if (_session_id.empty()) {
        _session_id = boost::lexical_cast<std::string>(boost::uuids::random_generator{}());
    }

    return _session_id;
}

} // namespace impl
} // namespace FLECS
