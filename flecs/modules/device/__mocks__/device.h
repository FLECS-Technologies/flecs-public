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

#include <gmock/gmock.h>

#include "flecs/modules/module_base/module.h"

namespace flecs {
namespace module {
namespace impl {
class device_t
{
public:
    ~device_t() = default;
};
} // namespace impl

class device_t FLECS_FINAL_UNLESS_TESTED : public base_t
{
    friend class factory_t;

public:
    ~device_t() = default;

    MOCK_METHOD((const std::string&), session_id, (), ());

    MOCK_METHOD((result_t), activate_license, (), ());
    MOCK_METHOD((result_t), validate_license, (), ());

protected:
    device_t() = default;

    MOCK_METHOD((void), do_init, (), (override));
    MOCK_METHOD((void), do_deinit, (), (override));

    MOCK_METHOD((result_t), do_load, (const fs::path&), (override));
    MOCK_METHOD((result_t), do_save, (const fs::path&), (const, override));

    std::unique_ptr<impl::device_t> _impl;
};

} // namespace module
} // namespace flecs
