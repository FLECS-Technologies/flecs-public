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

#ifndef B24999EC_55E9_4341_9915_0930AEC84E8F
#define B24999EC_55E9_4341_9915_0930AEC84E8F

#include "module_base/module.h"

namespace FLECS {

class module_usage_t : public module_t
{
public:
    module_usage_t();

private:
    http_status_e print_usage(const Json::Value& args, Json::Value& response);
};

} // namespace FLECS

#endif // B24999EC_55E9_4341_9915_0930AEC84E8F
