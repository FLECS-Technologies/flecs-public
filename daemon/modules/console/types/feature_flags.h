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

#include "util/json/json.h"

namespace flecs {
namespace console {

class feature_flags_t
{
public:
    auto is_vendor() const noexcept //
        -> bool;
    auto is_white_labeled() const noexcept //
        -> bool;

private:
    friend auto from_json(const json_t& j, feature_flags_t& ff) //
        -> void;
    friend auto to_json(json_t& j, const feature_flags_t& ff) //
        -> void;

    bool _is_vendor;
    bool _is_white_labeled;
};

} // namespace console
} // namespace flecs
