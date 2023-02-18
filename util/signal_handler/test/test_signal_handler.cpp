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

#include <sys/syscall.h>

#include <string>
#include <thread>

#include "gtest/gtest.h"
#include "util/signal_handler/signal_handler.h"

pid_t flecs_gettid()
{
    return syscall(SYS_gettid);
}

TEST(signal_handler, sigint)
{
    FLECS::signal_handler_init();

    ASSERT_FALSE(FLECS::g_stop);

    kill(flecs_gettid(), SIGINT);

    ASSERT_TRUE(FLECS::g_stop);
}

TEST(signal_handler, sigterm)
{
    FLECS::signal_handler_init();

    ASSERT_FALSE(FLECS::g_stop);

    kill(flecs_gettid(), SIGTERM);

    ASSERT_TRUE(FLECS::g_stop);
}

TEST(signal_handler, sigcont)
{
    FLECS::signal_handler_init();

    ASSERT_FALSE(FLECS::g_stop);

    kill(flecs_gettid(), SIGCONT);

    ASSERT_FALSE(FLECS::g_stop);
}
