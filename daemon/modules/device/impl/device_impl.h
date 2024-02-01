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

#pragma once

#include <string>

#include "daemon/modules/device/device.h"
#include "util/fs/fs.h"

namespace flecs {
namespace module {
namespace impl {

class device_t
{
    friend class flecs::module::device_t;

private:
    device_t(flecs::module::device_t* parent);

    auto do_init() //
        -> void;

    auto do_deinit() //
        -> void;

    auto do_load(const fs::path& base_path) //
        -> result_t;

    auto do_save(const fs::path& base_path) const //
        -> result_t;

    auto do_session_id() //
        -> const std::string&;

    auto do_activate_license() //
        -> result_t;

    auto do_validate_license() //
        -> result_t;

    auto do_activate_license_for_client() //
        -> crow::response;

    auto do_validate_license_for_client() //
        -> crow::response;

    flecs::module::device_t* _parent;
    std::string _session_id;
};

} // namespace impl
} // namespace module
} // namespace flecs
