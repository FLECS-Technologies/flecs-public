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

#include <map>
#include <memory>
#include <string>

#include "module_base/module.h"

namespace FLECS {

class module_factory_t
{
public:
    module_factory_t(const module_factory_t&) = delete;
    module_factory_t(module_factory_t&&) = delete;
    module_factory_t& operator=(module_factory_t) = delete;

    using module_table_t = std::map<std::string, std::shared_ptr<module_t>>;

    static module_factory_t& instance();

    template <typename T>
    void register_module(std::string module_name);

    void init_modules();
    void deinit_modules();
    std::shared_ptr<module_t> query(const std::string& endpoint);

private:
    module_factory_t() = default;

    module_table_t _module_table;
};

template <typename T>
void module_factory_t::register_module(std::string module_name)
{
    _module_table.try_emplace(module_name, new T{});
}

template <typename T>
class register_module_t
{
public:
    register_module_t(std::string module_name);
};

template <typename T>
register_module_t<T>::register_module_t(std::string module_name)
{
    module_factory_t::instance().register_module<T>(module_name);
}

namespace api {
void init_modules();
void deinit_modules();
std::shared_ptr<module_t> query_module(const std::string& module_name);
} // namespace api

} // namespace FLECS
