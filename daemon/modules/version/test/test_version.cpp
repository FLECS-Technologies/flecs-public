// Copyright 2021-2022 FLECS Technologies GmbH
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

#include "gtest/gtest.h"
#include "version/version.h"

class module_version_test_t : public FLECS::module_version_t
{
public:
    module_version_test_t() = default;
};

TEST(module_version, print_version)
{
    const auto out_expected = std::string{"{\"core\":\""} + FLECS_VERSION + "-" + FLECS_GIT_SHA + "\"}";

    auto mod = module_version_test_t{};
    auto response = nlohmann::json{};
    const auto res = mod.print_version(nlohmann::json{}, response);

    response.dump();

    ASSERT_EQ(res, FLECS::http_status_e::Ok);
    ASSERT_EQ(response.dump(), out_expected);
}
