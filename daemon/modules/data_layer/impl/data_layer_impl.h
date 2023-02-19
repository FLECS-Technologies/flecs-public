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

#include "data_layer/data_layer.h"
#include "flunder/flunder_client.h"

namespace FLECS {
namespace impl {

class module_data_layer_t
{
public:
    module_data_layer_t();

    ~module_data_layer_t();

    auto do_browse(std::string_view path) //
        -> crow::response;

    auto do_init() //
        -> void;

    auto do_deinit() //
        -> void;

private:
    FLECS::flunder_client_t _client;
};

} // namespace impl
} // namespace FLECS