// Copyright 2021-2024 FLECS Technologies GmbH
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

#include "flecs/modules/deployments/impl/deployments_impl.h"

#include "flecs/modules/deployments/types/deployment_docker.h"

namespace flecs {
namespace module {
namespace impl {

deployments_t::deployments_t()
    : _deployments{std::make_shared<deployments::docker_t>()}
{}

deployments_t::~deployments_t()
{}

auto deployments_t::do_module_load(const fs::path& base_path) //
    -> result_t
{
    auto result = result_t{};
    for (auto deployment : _deployments) {
        const auto [res, message] = deployment->load(base_path);
        if (res != 0) {
            std::get<0>(result) = res;
            std::get<1>(result).append(message);
        }
    }

    return result;
}

auto deployments_t::do_deployments() const noexcept //
    -> std::vector<std::shared_ptr<deployments::deployment_t>>
{
    return _deployments;
}

auto deployments_t::do_query_deployment(std::string_view id) //
    -> std::shared_ptr<deployments::deployment_t>
{
    auto it = std::find_if(
        _deployments.cbegin(),
        _deployments.cend(),
        [&id](decltype(_deployments)::const_reference elem) { return elem->deployment_id() == id; });

    if (it != _deployments.end()) {
        return *it;
    }

    return {};
}

} // namespace impl
} // namespace module
} // namespace flecs
