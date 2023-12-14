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

#include <vector>

#include "daemon/modules/jobs/types/job_id.h"
#include "daemon/modules/module_base/module.h"
#include "util/fs/fs.h"

namespace flecs {
namespace apps {
class key_t;
} // namespace apps

namespace instances {
class id_t;
} // namespace instances

namespace module {
namespace impl {
class flecsport_t
{
    ~flecsport_t() = default;
};
} // namespace impl

class flecsport_t FLECS_FINAL_UNLESS_TESTED : public base_t
{
public:
    ~flecsport_t() override = default;

    MOCK_METHOD((crow::response), http_list, (), ());
    MOCK_METHOD((crow::response), http_download, (const std::string&), ());
    MOCK_METHOD((crow::response), http_remove, (const std::string&), ());
    MOCK_METHOD(
        (crow::response), http_export_to, (std::vector<apps::key_t>, std::vector<instances::id_t>), ());
    MOCK_METHOD((crow::response), http_import_from, (std::string), ());

    MOCK_METHOD(
        (result_t), export_to, (std::vector<apps::key_t>, std::vector<instances::id_t>, fs::path), ());

protected:
    friend class factory_t;

    flecsport_t() = default;

    MOCK_METHOD((void), do_init, (), (override));
    MOCK_METHOD((void), do_deinit, (), (override));

    std::unique_ptr<impl::flecsport_t> _impl;
};

} // namespace module
} // namespace flecs
