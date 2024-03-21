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

#include "base_response.h"

namespace flecs {
namespace console {

class validate_response_data_t
{
public:
    auto is_valid() const noexcept //
        -> bool;

private:
    friend auto from_json(const json_t& j, validate_response_data_t& response) //
        -> void;
    friend auto to_json(json_t& j, const validate_response_data_t& response) //
        -> void;

    bool _is_valid;
};

class validate_response_t : public base_response_t, public validate_response_data_t
{
private:
    friend auto from_json(const json_t& j, validate_response_t& response) //
        -> void;
    friend auto to_json(json_t& j, const validate_response_t& response) //
        -> void;
};

} // namespace console
} // namespace flecs
