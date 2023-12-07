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

#include "module.h"

namespace flecs {
namespace module {

auto base_t::load(const fs::path& base_path) //
    -> result_t
{
    return do_load(base_path);
}

auto base_t::init() //
    -> void
{
    return do_init();
}

auto base_t::start() //
    -> void
{
    return do_start();
}

auto base_t::stop() //
    -> void
{
    return do_stop();
}

auto base_t::save(const fs::path& base_path) const //
    -> result_t
{
    return do_save(base_path);
}

auto base_t::deinit() //
    -> void
{
    return do_deinit();
}

auto base_t::do_load(const fs::path& /*base_path*/) //
    -> result_t
{
    return {0, {}};
}

auto base_t::do_start() //
    -> void
{}

auto base_t::do_stop() //
    -> void
{}

auto base_t::do_save(const fs::path& /*base_path*/) const //
    -> result_t
{
    return {0, {}};
}

} // namespace module
} // namespace flecs
