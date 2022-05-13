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

#ifndef E72B4073_69B1_4AE8_BCEC_210EBF10489E
#define E72B4073_69B1_4AE8_BCEC_210EBF10489E

#include "module_base/module.h"

namespace FLECS {

class module_system_t : public module_t
{
public:
    http_status_e ping(const nlohmann::json& args, nlohmann::json& response);

protected:
    friend class module_factory_t;

    module_system_t();

private:
};

} // namespace FLECS

#endif // E72B4073_69B1_4AE8_BCEC_210EBF10489E
