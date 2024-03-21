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
#include <list>
#include <mutex>
#include <optional>
#include <queue>
#include <thread>
#include <vector>

#include "flecs/modules/jobs/jobs.h"

namespace flecs {
namespace module {

class jobs_t;

namespace impl {
class jobs_t
{
    friend class flecs::module::jobs_t;

private:
    jobs_t();

    auto do_init() //
        -> void;

    auto do_deinit() //
        -> void;

    auto do_list_jobs(jobs::id_t job_id) const //
        -> crow::response;

    auto do_delete_job(jobs::id_t job_id) //
        -> crow::response;

    auto do_wait_for_job(jobs::id_t job_id) const //
        -> result_t;

    auto do_append(jobs::job_t job, std::string desc) //
        -> jobs::id_t;

    auto fetch_job() //
        -> std::optional<jobs::job_t>;

    auto worker_thread() //
        -> void;

    jobs::id_t _job_id;
    jobs::id_t _next_job_id;

    std::queue<jobs::job_t> _q;
    std::mutex _q_mutex;
    std::condition_variable _q_cv;

    std::list<jobs::progress_t> _job_progress;

    std::thread _worker_thread;
};

} // namespace impl
} // namespace module
} // namespace flecs
