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

#include "factory.h"

namespace FLECS {

module_factory_t& module_factory_t::instance()
{
    static module_factory_t factory;
    return factory;
}

void module_factory_t::init_modules()
{
    for (decltype(auto) it = _module_table.begin(); it != _module_table.end(); ++it) {
        it->second->init();
    }
}

void module_factory_t::deinit_modules()
{
    for (decltype(auto) it = _module_table.begin(); it != _module_table.end(); ++it) {
        it->second->deinit();
    }
}

std::shared_ptr<module_t> module_factory_t::query(const char* module_name)
{
    decltype(auto) it = _module_table.find(module_name);
    if (it != _module_table.end()) {
        return it->second;
    }
    return nullptr;
}

namespace api {
void init_modules()
{
    return module_factory_t::instance().init_modules();
}
void deinit_modules()
{
    return module_factory_t::instance().deinit_modules();
}
std::shared_ptr<module_t> query_module(const char* module_name)
{
    return module_factory_t::instance().query(module_name);
}
} // namespace api
} // namespace FLECS
