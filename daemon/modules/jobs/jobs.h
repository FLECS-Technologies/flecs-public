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

#include <cstdint>
#include <functional>
#include <memory>
#include <string>

#include "job_id.h"
#include "job_progress.h"
#include "module_base/module.h"

namespace FLECS {

namespace impl {
class module_jobs_t;
}

struct job_t
{
    std::function<result_t(job_progress_t&)> callable;
};

class module_jobs_t FLECS_FINAL_UNLESS_TESTED : public module_t
{
    friend class module_factory_t;

public:
    ~module_jobs_t();

    /** @brief Appends a new job to the queue
     *
     * @param[in] job
     * @param[in] progress
     *
     * @return job id assigned by api
     */
    auto append(job_t job, std::string desc) //
        -> job_id_t;

    auto list_jobs(job_id_t job_id) const //
        -> crow::response;

protected:
    module_jobs_t();

    auto do_init() //
        -> void override;

    auto do_deinit() //
        -> void override;

    std::unique_ptr<impl::module_jobs_t> _impl;
};

} // namespace FLECS
