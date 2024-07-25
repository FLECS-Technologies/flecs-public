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
#include <tuple>

#include "flecs/common/app/manifest/manifest.h"
#include "flecs/core/flecs.h"
#include "flecs/modules/module_base/module.h"
#include "flecs/util/fs/fs.h"
#include "flecs/util/json/json.h"

namespace flecs {
namespace apps {
class key_t;
} // namespace apps

class app_manifest_t;

namespace module {
namespace impl {

class manifests_t
{
    ~manifests_t() = default;
};

} // namespace impl

class manifests_t FLECS_FINAL_UNLESS_TESTED : public base_t
{
    friend class factory_t;

public:
    ~manifests_t() override = default;

    MOCK_METHOD((void), base_path, (const fs::path&), ());
    MOCK_METHOD((void), base_path, (), (const, noexcept));

    MOCK_METHOD((bool), migrate, (const fs::path&), ());
    MOCK_METHOD((bool), contains, (const apps::key_t&), (const, noexcept));

    MOCK_METHOD((std::shared_ptr<app_manifest_t>), query, (const apps::key_t&), (noexcept));
    MOCK_METHOD((std::shared_ptr<const app_manifest_t>), query, (const apps::key_t&), (const, noexcept));

    using add_result_t = std::tuple<std::shared_ptr<app_manifest_t>, bool>;
    MOCK_METHOD((add_result_t), add, (app_manifest_t), ());
    MOCK_METHOD((add_result_t), add_from_json, (const json_t&), ());

    MOCK_METHOD((add_result_t), add_from_string, (std::string_view), ());
    MOCK_METHOD((add_result_t), add_from_json_string, (std::string_view), ());

    MOCK_METHOD((add_result_t), add_from_file, (const fs::path&), ());
    MOCK_METHOD((add_result_t), add_from_json_file, (const fs::path&), ());

    MOCK_METHOD((add_result_t), add_from_console, (const apps::key_t&), ());
    MOCK_METHOD((add_result_t), add_from_url, (std::string_view), ());

    MOCK_METHOD((void), clear, (), ());

    MOCK_METHOD((void), erase, (const apps::key_t&), ());

    MOCK_METHOD((void), remove, (const apps::key_t&), ());

    MOCK_METHOD((fs::path), path, (const apps::key_t&), ());

protected:
    manifests_t() = default;

    MOCK_METHOD((void), do_init, (), (override));
    MOCK_METHOD((void), do_deinit, (), (override));

    std::unique_ptr<impl::manifests_t> _impl;
};

} // namespace module
} // namespace flecs
