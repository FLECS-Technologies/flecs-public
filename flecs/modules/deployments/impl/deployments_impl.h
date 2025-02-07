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

#include "flecs/modules/deployments/deployments.h"

namespace flecs {
namespace module {
namespace impl {

class deployments_t
{
    friend class flecs::module::deployments_t;

public:
    ~deployments_t();

private:
    deployments_t();

    auto do_module_load(const fs::path& base_path) //
        -> result_t;

    auto do_deployments() const noexcept //
        -> std::vector<std::shared_ptr<deployments::deployment_t>>;

    auto do_query_deployment(std::string_view id) //
        -> std::shared_ptr<deployments::deployment_t>;

    std::vector<std::shared_ptr<deployments::deployment_t>> _deployments;
};

} // namespace impl
} // namespace module
} // namespace flecs
