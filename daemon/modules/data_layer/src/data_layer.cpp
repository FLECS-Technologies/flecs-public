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

#include "data_layer.h"

#include "factory/factory.h"
#include "impl/data_layer_impl.h"

namespace FLECS {

namespace {
register_module_t<module_data_layer_t> _reg("data-layer");
}

module_data_layer_t::module_data_layer_t()
    : _impl{new impl::module_data_layer_t{}}
{}

module_data_layer_t::~module_data_layer_t()
{}

auto module_data_layer_t::do_init() //
    -> void
{
    _impl->do_init();

    FLECS_ROUTE("/data-layer/browse").methods("GET"_method)([]() {
        auto response = crow::response{};
        response.moved_perm("/v2/data-layer/browse");
        return response;
    });

    FLECS_V2_ROUTE("/data-layer/browse").methods("GET"_method)([=]() { return browse(""); });
}

auto module_data_layer_t::do_deinit() //
    -> void
{
    _impl->do_deinit();
}

auto module_data_layer_t::browse(std::string_view path) //
    -> crow::response
{
    return _impl->do_browse(std::move(path));
}

} // namespace FLECS
