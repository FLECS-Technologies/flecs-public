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

#include <condition_variable>
#include <cstdint>
#include <functional>
#include <memory>
#include <mutex>
#include <string>

#include "daemon/modules/module_base/module.h"
#include "types.h"

namespace flecs {
namespace module {
namespace impl {
class jobs_t;
} // namespace impl

class jobs_t FLECS_FINAL_UNLESS_TESTED : public base_t
{
    friend class factory_t;

public:
    ~jobs_t();

    /** @brief Appends a new job to the queue
     *
     * @param[in] job
     * @param[in] progress
     *
     * @return job id assigned by api
     */
    auto append(jobs::job_t job, std::string desc) //
        -> jobs::id_t;

    auto list_jobs(jobs::id_t job_id) const //
        -> crow::response;

    auto delete_job(jobs::id_t job_id) //
        -> crow::response;

    auto wait_for_job(jobs::id_t job_id) const //
        -> result_t;

protected:
    jobs_t();

    auto do_init() //
        -> void override;

    auto do_deinit() //
        -> void override;

    std::unique_ptr<impl::jobs_t> _impl;
};

} // namespace module
} // namespace flecs
