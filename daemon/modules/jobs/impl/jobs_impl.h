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
#include <mutex>
#include <optional>
#include <queue>
#include <set>
#include <thread>
#include <vector>

#include "jobs.h"

namespace FLECS {

class module_jobs_t;

namespace impl {

class module_jobs_t
{
    friend class FLECS::module_jobs_t;

private:
    module_jobs_t();

    auto do_init() //
        -> void;

    auto do_deinit() //
        -> void;

    auto do_append(job_t job) //
        -> job_id_t;

    auto wait_for_job() //
        -> std::optional<job_t>;

    auto worker_thread() //
        -> void;

    job_id_t _job_id;
    job_id_t _active_job_id;

    std::mutex _q_mutex;
    std::queue<job_t> _q;
    std::condition_variable _q_cv;
    std::set<job_progress_t> _job_progress;

    std::thread _worker_thread;
};

} // namespace impl
} // namespace FLECS
