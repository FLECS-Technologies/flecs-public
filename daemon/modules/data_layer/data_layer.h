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

#include <memory>

#include "module_base/module.h"

namespace FLECS {
namespace module {

namespace impl {
class data_layer_t;
} // namespace impl

class data_layer_t FLECS_FINAL_UNLESS_TESTED : public base_t
{
    friend class factory_t;

public:
    ~data_layer_t() override;

protected:
    data_layer_t();

    auto do_init() //
        -> void override;
    auto do_deinit() //
        -> void override;

    auto browse(std::string_view path) //
        -> crow::response;

#if 0
    /* add persistent in-memory storage for path */
    int add_mem_storage(const std::string_view& path);

    /* remove persistent in-memory storage for path */
    int remove_mem_storage(const std::string_view& path);
#endif // 0

private:
    std::unique_ptr<impl::data_layer_t> _impl;
};

} // namespace module
} // namespace FLECS
