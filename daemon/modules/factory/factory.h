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

#ifndef FLECS_daemon_modules_factory_h
#define FLECS_daemon_modules_factory_h

#include <map>
#include <memory>

#include "module_base/module.h"
#include "util/string/comparator.h"

namespace FLECS {

namespace {
template <typename T>
auto make_module()
{
    return std::shared_ptr<module_t>{std::make_shared<T>()};
}
} // namespace

class module_factory_t
{
public:
    module_factory_t(const module_factory_t&) = delete;
    module_factory_t(module_factory_t&&) = delete;
    module_factory_t& operator=(module_factory_t) = delete;

    using module_table_t = std::map<const char*, std::shared_ptr<module_t>, string_comparator_t>;

    static module_factory_t& instance();

    template <typename T>
    void register_module(const char* module_name);

    std::shared_ptr<module_t> query(const char* endpoint);

private:
    friend void ctor_init_modules();
    void init_modules();

    module_factory_t() = default;

    module_table_t _module_table;
};

template <typename T>
void module_factory_t::register_module(const char* module_name)
{
    _module_table.try_emplace(module_name, make_module<T>());
}

template <typename T>
class register_module_t
{
public:
    register_module_t(const char* module_name);
};

template <typename T>
register_module_t<T>::register_module_t(const char* module_name)
{
    module_factory_t::instance().register_module<T>(module_name);
}

namespace api {
std::shared_ptr<module_t> query_module(const char* module_name);
} // namespace api

} // namespace FLECS

#endif // FLECS_daemon_modules_factory_h
