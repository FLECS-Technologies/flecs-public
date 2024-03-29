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

#include "flecs/modules/data_layer/data_layer.h"
#include "flunder/client.h"

namespace flecs {
namespace module {
namespace impl {

class data_layer_t
{
public:
    data_layer_t();

    ~data_layer_t();

    auto do_browse(std::string_view path) //
        -> crow::response;

    auto do_init() //
        -> void;

    auto do_deinit() //
        -> void;

private:
    flunder::client_t _client;
};

} // namespace impl
} // namespace module
} // namespace flecs
