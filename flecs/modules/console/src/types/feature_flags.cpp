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

#include "flecs/modules/console/types/feature_flags.h"

namespace flecs {
namespace console {

feature_flags_t::feature_flags_t()
    : _is_vendor{}
    , _is_white_labeled()
{}

auto feature_flags_t::is_vendor() const noexcept //
    -> bool
{
    return _is_vendor;
}

auto feature_flags_t::is_white_labeled() const noexcept //
    -> bool
{
    return _is_white_labeled;
}

auto from_json(const json_t& j, feature_flags_t& ff) //
    -> void
{
    j.at("isVendor").get_to(ff._is_vendor);
    j.at("isWhitelabeled").get_to(ff._is_white_labeled);
}

auto to_json(json_t& j, const feature_flags_t& ff) //
    -> void
{
    j = json_t({
        {"isVendor", ff.is_vendor()},
        {"isWhitelabeled", ff.is_white_labeled()},
    });
}

} // namespace console
} // namespace flecs
