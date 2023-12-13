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

#include <gtest/gtest.h>
#include <sys/syscall.h>

#include <condition_variable>
#include <mutex>

#include "daemon/modules/jobs/jobs.h"
#include "util/signal_handler/signal_handler.h"

class test_module_jobs_t : public flecs::module::jobs_t
{
};

pid_t flecs_gettid()
{
    return syscall(SYS_gettid);
}

TEST(jobs, empty)
{
    flecs::signal_handler_init();

    auto uut = test_module_jobs_t{};
    uut.init();
    /* make sure nothing bad happens during idle... */
    std::this_thread::sleep_for(std::chrono::milliseconds(100));
    kill(flecs_gettid(), SIGTERM);
    uut.deinit();
}

struct job_t
{
    std::condition_variable cv = {};
    std::mutex mutex = {};
    bool executed = {};

    flecs::result_t test_func(flecs::jobs::id_t, int, flecs::jobs::progress_t&);
};

flecs::result_t job_t::test_func(
    flecs::jobs::id_t job_id, int exit_code, flecs::jobs::progress_t& progress)
{
    auto lock = std::lock_guard{mutex};
    executed = true;
    cv.notify_one();

    if (progress.job_id() != job_id) {
        return {-1, {}};
    }

    return {exit_code, {}};
}

auto make_job()
{
    return job_t{};
}

TEST(jobs, schedule)
{
    flecs::signal_handler_init();

    auto job = job_t{};

    auto uut = test_module_jobs_t{};
    uut.init();

    uut.append(
        flecs::jobs::job_t{
            std::bind(&job_t::test_func, &job, flecs::jobs::id_t{1}, 0, std::placeholders::_1)},
        "FLECS unit test job");

    auto lock = std::unique_lock{job.mutex};
    ASSERT_TRUE(job.cv.wait_for(lock, std::chrono::seconds(1), [&job]() { return job.executed; }));
    kill(flecs_gettid(), SIGTERM);
    uut.deinit();

    ASSERT_TRUE(job.executed);
}

TEST(jobs, status)
{
    flecs::signal_handler_init();

    auto job_0 = job_t{};
    auto job_1 = job_t{};

    auto uut = test_module_jobs_t{};
    uut.init();

    uut.append(
        flecs::jobs::job_t{
            std::bind(&job_t::test_func, &job_0, flecs::jobs::id_t{1}, 0, std::placeholders::_1)},
        "FLECS unit test job");
    uut.append(
        flecs::jobs::job_t{
            std::bind(&job_t::test_func, &job_1, flecs::jobs::id_t{2}, -1, std::placeholders::_1)},
        "FLECS unit test job");

    {
        const auto [res, message] = uut.wait_for_job(0);
        ASSERT_EQ(res, -1);
    }
    {
        const auto [res, message] = uut.wait_for_job(1);
        ASSERT_EQ(res, 0);
        ASSERT_TRUE(job_0.executed);
    }
    {
        auto lock = std::unique_lock{job_1.mutex};
        ASSERT_TRUE(job_1.cv.wait_for(lock, std::chrono::seconds(1), [&job_1]() {
            return job_1.executed;
        }));
    }
    {
        const auto [res, message] = uut.wait_for_job(2);
        ASSERT_EQ(res, -1);
        ASSERT_TRUE(job_1.executed);
    }

    kill(flecs_gettid(), SIGTERM);
    uut.deinit();

    ASSERT_TRUE(job_0.executed);
    ASSERT_EQ(
        flecs::parse_json(uut.list_jobs(flecs::jobs::id_t{1}).body)[0]["status"],
        "successful");

    ASSERT_TRUE(job_1.executed);
    ASSERT_EQ(flecs::parse_json(uut.list_jobs(flecs::jobs::id_t{2}).body)[0]["status"], "failed");
}
