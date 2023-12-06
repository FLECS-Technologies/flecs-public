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

#include <algorithm>

#include "jobs.h"
#include "util/signal_handler/signal_handler.h"

namespace FLECS {
namespace module {
namespace impl {

jobs_t::jobs_t()
    : _job_id{}
    , _next_job_id{}
    , _q{}
    , _q_mutex{}
    , _q_cv{}
    , _job_progress{}
    , _worker_thread{}
{}

auto jobs_t::do_init() //
    -> void
{
    _worker_thread = std::thread{&jobs_t::worker_thread, this};
}

auto jobs_t::do_deinit() //
    -> void
{
    _worker_thread.join();
}

auto jobs_t::do_append(job_t job, std::string desc) //
    -> job_id_t
{
    {
        auto lock = std::lock_guard{_q_mutex};
        _q.push(std::move(job));
    }
    _job_progress.emplace_back(++_job_id, std::move(desc));
    _q_cv.notify_one();
    return _job_id;
}

auto jobs_t::do_delete_job(job_id_t job_id) //
    -> crow::response
{
    auto job = std::find_if(
        _job_progress.begin(),
        _job_progress.end(),
        [&job_id](const job_progress_t& elem) { return elem.job_id() == job_id; });

    if (job == _job_progress.end()) {
        return crow::response{
            crow::status::NOT_FOUND,
            "txt",
            "No such job " + std::to_string(job_id)};
    }

    auto status = job->status();
    if (status == job_status_e::Cancelled || status == job_status_e::Successful ||
        status == job_status_e::Failed) {
        _job_progress.erase(job);
        return crow::response{crow::status::NO_CONTENT};
    }

    return {
        crow::status::BAD_REQUEST,
        "txt",
        "Not removing unfinished job " + std::to_string(job_id)};
}

auto jobs_t::do_list_jobs(job_id_t job_id) const //
    -> crow::response
{
    auto response = json_t::array();

    for (const auto& progress : _job_progress) {
        if ((job_id != job_id_t{}) && (job_id != progress.job_id())) {
            continue;
        }
        response.push_back(progress);
    }

    if ((job_id != job_id_t{}) && response.empty()) {
        return crow::response{
            crow::status::NOT_FOUND,
            "txt",
            "No such job " + std::to_string(job_id)};
    }

    return crow::response{crow::status::OK, "json", response.dump()};
}

auto jobs_t::do_wait_for_job(job_id_t job_id) const //
    -> result_t
{
    if (job_id == job_id_t{}) {
        return {-1, "Empty job_id specified"};
    }

    const auto it = std::find_if(
        _job_progress.cbegin(),
        _job_progress.cend(),
        [&job_id](const job_progress_t& elem) { return elem.job_id() == job_id; });

    if (it == _job_progress.cend()) {
        return {-1, "No such job " + std::to_string(job_id)};
    }

    do {
        auto status = it->status();
        if (status == job_status_e::Cancelled || status == job_status_e::Successful ||
            status == job_status_e::Failed) {
            break;
        }
        std::this_thread::sleep_for(std::chrono::milliseconds(10));
    } while (true);

    return it->result();
}

auto jobs_t::fetch_job() //
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

auto jobs_t::worker_thread() //
    -> void
{
    pthread_setname_np(pthread_self(), "job_scheduler");

    while (!g_stop) {
        auto job = fetch_job();
        if (!job.has_value()) {
            continue;
        }

        ++_next_job_id;
        auto job_thread = std::thread{[this, job = std::move(job)]() {
            char thread_name[16] = {};
            snprintf(thread_name, sizeof(thread_name) - 1, "job_%d", _next_job_id);
            pthread_setname_np(pthread_self(), thread_name);
            auto& job_progress = *std::find_if(
                _job_progress.begin(),
                _job_progress.end(),
                [this](job_progress_t& item) { return item.job_id() == _next_job_id; });

            job_progress.status(job_status_e::Running);
            auto [code, message] = std::invoke(job->callable(), job_progress);
            job_progress.result(code, std::move(message));
            job_progress.status(code == 0 ? job_status_e::Successful : job_status_e::Failed);
        }};
        job_thread.join();
    }
}

} // namespace impl
} // namespace module
} // namespace FLECS
