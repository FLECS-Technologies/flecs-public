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

#include "flecs/modules/module_base/module.h"

namespace flecs {
namespace module {

class version_t FLECS_FINAL_UNLESS_TESTED : public base_t
{
    friend class factory_t;

public:
    auto core_version() const //
        -> std::string;

    auto api_version() const //
        -> std::string;

protected:
    version_t();

    auto do_init() //
        -> void override;
    auto do_deinit() //
        -> void override;
};

} // namespace module
} // namespace flecs
