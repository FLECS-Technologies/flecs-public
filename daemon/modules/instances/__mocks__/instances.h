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

#include <gmock/gmock.h>

#include <memory>
#include <string>
#include <vector>

#include "daemon/common/app/app_key.h"
#include "daemon/common/instance/instance_id.h"
#include "daemon/modules/module_base/module.h"
#include "util/fs/fs.h"

namespace flecs {

class app_key_t;
class instance_config_t;
class instance_t;

namespace module {
namespace impl {
class instances_t
{
    ~instances_t() = default;
};
} // namespace impl

class instances_t : public base_t
{
public:
    ~instances_t() override = default;

    MOCK_METHOD((crow::response), http_list, (const app_key_t&), (const));
    MOCK_METHOD((crow::response), http_details, (instance_id_t), (const));
    MOCK_METHOD((crow::response), http_create, (app_key_t, std::string, bool), ());
    MOCK_METHOD((crow::response), http_start, (instance_id_t), ());
    MOCK_METHOD((crow::response), http_stop, (instance_id_t), ());
    MOCK_METHOD((crow::response), http_remove, (instance_id_t), ());
    MOCK_METHOD((crow::response), http_get_config, (instance_id_t), (const));
    MOCK_METHOD(
        (crow::response), http_post_config, (instance_id_t, const instance_config_t&), (const));
    MOCK_METHOD((crow::response), http_logs, (instance_id_t), (const));
    MOCK_METHOD((crow::response), http_update, (instance_id_t, std::string));
    MOCK_METHOD((crow::response), http_export_to, (instance_id_t, fs::path), (const));

    MOCK_METHOD((std::vector<instance_id_t>), instance_ids, (const app_key_t&), (const)); //
    MOCK_METHOD((std::vector<instance_id_t>), instance_ids, (std::string, std::string), (const));
    MOCK_METHOD((std::vector<instance_id_t>), instance_ids, (std::string), (const));
    MOCK_METHOD((std::vector<instance_id_t>), instance_ids, (), (const));

    MOCK_METHOD((std::shared_ptr<instance_t>), query, (instance_id_t), (const));
    MOCK_METHOD((bool), is_running, (std::shared_ptr<instance_t>), (const));

    MOCK_METHOD((result_t), create, (app_key_t, std::string, bool), ());
    MOCK_METHOD((result_t), create, (app_key_t), ());
    MOCK_METHOD((result_t), create, (app_key_t, std::string, std::string), ());
    MOCK_METHOD((result_t), create, (std::string, std::string), ());

    MOCK_METHOD((result_t), start, (instance_id_t), ());
    MOCK_METHOD((result_t), start_once, (instance_id_t), ());

    MOCK_METHOD((result_t), stop, (instance_id_t), ());
    MOCK_METHOD((result_t), stop_once, (instance_id_t), ());

    MOCK_METHOD((result_t), remove, (instance_id_t), ());

    MOCK_METHOD((result_t), export_to, (instance_id_t, fs::path), (const));

    MOCK_METHOD((result_t), import_from, (instance_id_t, fs::path), ());

protected:
    friend class factory_t;

    instances_t() = default;

    MOCK_METHOD((void), do_init, (), (override));
    MOCK_METHOD((void), do_deinit, (), (override));

    MOCK_METHOD((result_t), do_load, (const fs::path&), (override));
    MOCK_METHOD((void), do_start, (), (override));
    MOCK_METHOD((void), do_stop, (), (override));

    std::unique_ptr<impl::instances_t> _impl;
};

} // namespace module
} // namespace flecs
