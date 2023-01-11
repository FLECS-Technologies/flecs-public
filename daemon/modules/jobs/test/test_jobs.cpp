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

class test_module_jobs_t : public FLECS::module_jobs_t
{
};

pid_t flecs_gettid()
{
    return syscall(SYS_gettid);
}

TEST(jobs, empty)
{
    FLECS::signal_handler_init();

    auto uut = test_module_jobs_t{};
    uut.init();
    /* make sure nothing bad happens during idle... */
    std::this_thread::sleep_for(std::chrono::milliseconds(100));
    kill(flecs_gettid(), SIGTERM);
    uut.deinit();
}

TEST(jobs, schedule)
{
    auto cv = std::condition_variable{};
    auto mutex = std::mutex{};

    FLECS::signal_handler_init();

    auto uut = test_module_jobs_t{};
    uut.init();

    auto executed = false;
    auto test_func = [&executed, &cv, &mutex](FLECS::job_progress_t& progress) {
        ASSERT_EQ(progress.job_id(), 1);
        auto lock = std::lock_guard{mutex};
        executed = true;
        cv.notify_one();
    };

    uut.append(FLECS::job_t{test_func});

    auto lock = std::unique_lock{mutex};
    ASSERT_TRUE(cv.wait_for(lock, std::chrono::seconds(5), [&executed]() { return executed; }));
    kill(flecs_gettid(), SIGTERM);
    uut.deinit();

    ASSERT_TRUE(executed);
}
