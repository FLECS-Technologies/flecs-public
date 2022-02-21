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

TEST(module_version, print_version)
{
    const auto out_expected = std::string{"{\n\t\"core\" : \""} + FLECS_VERSION + "-" + FLECS_GIT_SHA + "\"\n}\n";

    auto mod = FLECS::module_version_t{};
    auto response = Json::Value{};
    const auto res = mod.print_version(Json::Value{}, response);

    response.toStyledString();

    ASSERT_EQ(res, FLECS::http_status_e::Ok);
    ASSERT_EQ(response.toStyledString(), out_expected);
}
