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
#include "system/system.h"

class module_system_test_t : public FLECS::module_system_t
{
public:
    module_system_test_t() = default;
};

TEST(module_version, ping)
{
    const auto out_expected = std::string{"{\"additionalInfo\":\"OK\"}"};

    auto mod = module_system_test_t{};
    auto response = json_t{};
    const auto res = mod.ping(json_t{}, response);

    response.dump();

    ASSERT_EQ(res, FLECS::http_status_e::Ok);
    ASSERT_EQ(response.dump(), out_expected);
}
