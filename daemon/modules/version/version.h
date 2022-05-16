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

class module_version_t : public module_t
{
public:
    http_status_e print_version(const json_t& args, json_t& response);

protected:
    friend class module_factory_t;

    module_version_t();
};

} // namespace FLECS

#endif // A120ECB5_AF1A_49CB_8C2E_6A09CF6F242C
