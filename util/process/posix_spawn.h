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

#include <spawn.h>
#include <unistd.h>

extern char** environ;

namespace flecs {

class flecs_posix_spawn_file_actions_t
{
public:
    flecs_posix_spawn_file_actions_t() { posix_spawn_file_actions_init(&_file_actions); }

    ~flecs_posix_spawn_file_actions_t() { posix_spawn_file_actions_destroy(&_file_actions); }

    auto pointer() noexcept -> posix_spawn_file_actions_t* { return &_file_actions; }

private:
    posix_spawn_file_actions_t _file_actions;
};

class flecs_posix_spawnattr_t
{
public:
    flecs_posix_spawnattr_t() { posix_spawnattr_init(&_attr); }

    ~flecs_posix_spawnattr_t() { posix_spawnattr_destroy(&_attr); }

    auto pointer() noexcept -> posix_spawnattr_t* { return &_attr; }

private:
    posix_spawnattr_t _attr;
};

} // namespace flecs
