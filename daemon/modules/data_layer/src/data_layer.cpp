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

#include "daemon/modules/data_layer/data_layer.h"

#include "daemon/modules/data_layer/impl/data_layer_impl.h"
#include "daemon/modules/factory/factory.h"

namespace flecs {
namespace module {

namespace {
register_module_t<data_layer_t> _reg("data-layer");
}

data_layer_t::data_layer_t()
    : _impl{new impl::data_layer_t{}}
{}

data_layer_t::~data_layer_t()
{}

auto data_layer_t::do_init() //
    -> void
{
    _impl->do_init();

    FLECS_V2_ROUTE("/data-layer/browse").methods("GET"_method)([this]() { return browse(""); });
}

auto data_layer_t::do_deinit() //
    -> void
{
    _impl->do_deinit();
}

auto data_layer_t::browse(std::string_view path) //
    -> crow::response
{
    return _impl->do_browse(std::move(path));
}

} // namespace module
} // namespace flecs
