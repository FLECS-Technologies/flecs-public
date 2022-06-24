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

#include "data_layer.h"

#include "factory/factory.h"
#include "private/data_layer_private.h"

namespace FLECS {

namespace {
register_module_t<module_data_layer_t> _reg("data-layer");
}

module_data_layer_t::module_data_layer_t()
    : _impl{new Private::module_data_layer_private_t{}}
{}

module_data_layer_t::~module_data_layer_t()
{}

auto module_data_layer_t::do_init() //
    -> void
{
    FLECS_ROUTE("/data-layer/browse").methods("GET"_method)([=]() {
        auto response = json_t{};
        const auto status = _impl->do_browse("", response);
        return crow::response{status, response.dump()};
    });
}

} // namespace FLECS
