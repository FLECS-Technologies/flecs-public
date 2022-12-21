// Copyright 2021-2022 FLECS Technologies GmbH
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

#ifndef A120ECB5_AF1A_49CB_8C2E_6A09CF6F242C
#define A120ECB5_AF1A_49CB_8C2E_6A09CF6F242C

#include "module_base/module.h"

namespace FLECS {

class module_version_t FLECS_FINAL_UNLESS_TESTED : public module_t
{
    friend class module_factory_t;

protected:
    module_version_t();

    auto do_init() //
        -> void override;
    auto do_deinit() //
        -> void override;

    auto version() const //
        -> crow::response;
};

} // namespace FLECS

#endif // A120ECB5_AF1A_49CB_8C2E_6A09CF6F242C
