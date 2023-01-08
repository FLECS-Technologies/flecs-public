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

#include "impl/jobs_impl.h"

#include "jobs.h"
#include "util/signal_handler/signal_handler.h"

namespace FLECS {
namespace impl {

module_jobs_t::module_jobs_t()
    : _job_id{}
    , _active_job_id{}
    , _q_mutex{}
    , _q{}
    , _q_cv{}
    , _job_progress{}
    , _worker_thread{}
{}

auto module_jobs_t::do_init() //
    -> void
{
    _worker_thread = std::thread{&module_jobs_t::worker_thread, this};
}

auto module_jobs_t::do_deinit() //
    -> void
{
    _worker_thread.join();
}

auto module_jobs_t::do_append(job_t job) //
    -> job_id_t
{
    {
        auto lock = std::lock_guard{_q_mutex};
        _q.push(std::move(job));
    }
    auto job_progress = job_progress_t{++_job_id};
    _job_progress.insert(std::move(job_progress));
    _q_cv.notify_one();
    return _job_id;
}

auto module_jobs_t::wait_for_job() //
    -> std::optional<job_t>
{
    auto lock = std::unique_lock{_q_mutex};
    if (!_q_cv.wait_for(lock, std::chrono::milliseconds(10), [this]() { return !_q.empty(); })) {
        return {};
    }

    auto job = std::move(_q.front());
    _q.pop();
    return job;
}

auto module_jobs_t::worker_thread() //
    -> void
{
    pthread_setname_np(pthread_self(), "job_scheduler");

    while (!g_stop) {
        auto job = wait_for_job();
        if (!job.has_value()) {
            continue;
        }

        ++_active_job_id;
        auto job_thread = std::thread{[this, job = std::move(job)]() {
            char thread_name[16] = {};
            snprintf(thread_name, sizeof(thread_name) - 1, "job_%d", _active_job_id);
            pthread_setname_np(pthread_self(), thread_name);
            std::invoke(job->callable);
        }};
        job_thread.join();
    }
}

} // namespace impl
} // namespace FLECS
