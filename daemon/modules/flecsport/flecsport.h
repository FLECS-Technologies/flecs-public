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
class flecsport_t;
} // namespace impl

class flecsport_t FLECS_FINAL_UNLESS_TESTED : public base_t
{
public:
    ~flecsport_t() override;

    auto http_list() //
        -> crow::response;

    auto http_download(const std::string& export_id) //
        -> crow::response;

    auto http_remove(const std::string& export_id) //
        -> crow::response;

    auto http_export_to(std::vector<apps::key_t> apps, std::vector<instances::id_t> instances) //
        -> crow::response;

    auto http_import_from(std::string archive) //
        -> crow::response;

    auto export_to(
        std::vector<apps::key_t> apps, std::vector<instances::id_t> instances, fs::path base_dir) //
        -> result_t;

protected:
    friend class factory_t;

    flecsport_t();

    auto do_init() //
        -> void override;

    auto do_deinit() //
        -> void override
    {}

    std::unique_ptr<impl::flecsport_t> _impl;
};

} // namespace module
} // namespace flecs
