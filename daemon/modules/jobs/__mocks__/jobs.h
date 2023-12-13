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

#include "daemon/modules/jobs/types.h"
#include "daemon/modules/module_base/module.h"

namespace flecs {
namespace module {

namespace impl {

class jobs_t
{
    ~jobs_t() = default;
};

} // namespace impl

class jobs_t FLECS_FINAL_UNLESS_TESTED : public base_t
{
    friend class factory_t;

public:
    ~jobs_t() = default;

    MOCK_METHOD((jobs::id_t), append, (jobs::job_t, std::string), ());
    MOCK_METHOD((crow::response), list_jobs, (jobs::id_t), (const));
    MOCK_METHOD((crow::response), delete_job, (jobs::id_t), ());
    MOCK_METHOD((result_t), wait_for_job, (jobs::id_t), (const));

protected:
    jobs_t() = default;

    MOCK_METHOD((void), do_init, (), (override));
    MOCK_METHOD((void), do_deinit, (), (override));

    std::unique_ptr<impl::jobs_t> _impl;
};

} // namespace module
} // namespace flecs
