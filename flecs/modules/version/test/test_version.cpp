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

#include "flecs/modules/version/version.h"
#include "flecs/util/json/json.h"

class test_module_version_t : public flecs::module::version_t
{
public:
    test_module_version_t() = default;

    auto do_init() //
        -> void override
    {
        return flecs::module::version_t::do_init();
    }

    auto do_deinit() //
        -> void override
    {
        return flecs::module::version_t::do_deinit();
    }
};

static auto uut = test_module_version_t{};

TEST(module_version, init)
{
    uut.do_init();

    flecs::flecs_api_t::instance().app().validate();
}

TEST(module_version, deinit)
{
    uut.do_deinit();
}
