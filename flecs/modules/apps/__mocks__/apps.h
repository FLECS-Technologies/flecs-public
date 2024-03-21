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

#include "flecs/modules/apps/types.h"
#include "flecs/modules/module_base/module.h"
#include "flecs/util/fs/fs.h"

namespace flecs {
namespace apps {
class app_t;
} // namespace apps

class app_manifest_t;

namespace module {
namespace impl {
class apps_t
{
    ~apps_t() = default;
};
} // namespace impl

class apps_t FLECS_FINAL_UNLESS_TESTED : public base_t
{
public:
    ~apps_t() override = default;

    MOCK_METHOD((result_t), do_load, (const fs::path&), (override));

    MOCK_METHOD((void), do_start, (), (override));

    MOCK_METHOD((result_t), do_save, (const fs::path&), (const, override));

    MOCK_METHOD((crow::response), http_list, (const apps::key_t&), (const));
    MOCK_METHOD((crow::response), http_install, (apps::key_t), ());
    MOCK_METHOD((crow::response), http_sideload, (std::string), ());
    MOCK_METHOD((crow::response), http_uninstall, (apps::key_t), ());
    MOCK_METHOD((crow::response), http_export_to, (apps::key_t), ());

    MOCK_METHOD((std::vector<apps::key_t>), app_keys, (const apps::key_t&), (const));
    MOCK_METHOD((std::vector<apps::key_t>), app_keys, (std::string, std::string), (const));
    MOCK_METHOD((std::vector<apps::key_t>), app_keys, (std::string), (const));
    MOCK_METHOD((std::vector<apps::key_t>), app_keys, (), (const));

    MOCK_METHOD((std::shared_ptr<apps::app_t>), query, (const apps::key_t&), (const, noexcept));

    MOCK_METHOD((result_t), install_from_marketplace, (apps::key_t), ());

    MOCK_METHOD((result_t), sideload, (std::string), ());

    MOCK_METHOD((result_t), uninstall, (apps::key_t), ());

    MOCK_METHOD((result_t), export_to, (apps::key_t, fs::path), ());

    MOCK_METHOD((result_t), import_from, (apps::key_t, fs::path), ());

    MOCK_METHOD((bool), is_installed, (const apps::key_t&), (const, noexcept));

protected:
    friend class factory_t;

    apps_t() = default;

    MOCK_METHOD((void), do_init, (), (override));
    MOCK_METHOD((void), do_deinit, (), (override));

    std::unique_ptr<impl::apps_t> _impl;
};

} // namespace module
} // namespace flecs
