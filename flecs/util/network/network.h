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

#include <arpa/inet.h>
#include <netinet/in.h>

#include <map>
#include <string>

#include "cxxbridge/flecs_core_cxx_bridge/src/lib.rs.h"

namespace flecs {

auto subnet_mask_to_cidr_v4(std::string_view subnet_mask) //
    -> std::size_t;
auto cidr_to_subnet_mask_v4(std::string_view cidr_subnet) //
    -> std::string;

auto ipv4_to_network(std::string_view ip, std::string_view subnet_mask) //
    -> std::string;

auto get_network_adapters() //
    -> std::map<std::string, NetInfo>;

} // namespace flecs
