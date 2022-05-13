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

#ifndef C747A5B8_3554_4BE3_B178_63AA7227B37D
#define C747A5B8_3554_4BE3_B178_63AA7227B37D

#include <memory>

#include "module_base/module.h"

namespace FLECS {

namespace Private {
class module_data_layer_private_t;
} // namespace Private

class module_data_layer_t : public module_t
{
public:
    ~module_data_layer_t() override;

    http_status_e browse(const nlohmann::json& args, nlohmann::json& response);
#if 0
    /* add persistent in-memory storage for path */
    int add_mem_storage(const std::string_view& path);

    /* remove persistent in-memory storage for path */
    int remove_mem_storage(const std::string_view& path);
#endif // 0
protected:
    friend class module_factory_t;

    module_data_layer_t();

private:
    std::unique_ptr<Private::module_data_layer_private_t> _impl;
};

} // namespace FLECS

#endif // C747A5B8_3554_4BE3_B178_63AA7227B37D
