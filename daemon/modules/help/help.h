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

#ifndef ADE73181_EBFC_4CF5_B1EE_F5FD36BFAAA4
#define ADE73181_EBFC_4CF5_B1EE_F5FD36BFAAA4

#include <list>
#include <string>

#include "module_base/module.h"

namespace FLECS {

class module_help_t : public module_t
{
public:
private:
    friend class module_factory_t;

    module_help_t();

    http_status_e print_help(const Json::Value& args);

    std::string _topic;
    std::list<std::string> _subtopics;
};

} // namespace FLECS

#endif // ADE73181_EBFC_4CF5_B1EE_F5FD36BFAAA4
