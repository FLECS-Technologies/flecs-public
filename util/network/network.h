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

#ifndef F34AD8B9_5FE1_4B5D_A309_BBE14AD32A7A
#define F34AD8B9_5FE1_4B5D_A309_BBE14AD32A7A

#include <arpa/inet.h>
#include <netinet/in.h>

#include <string>

namespace FLECS {

in_addr ipv4_to_bits(const std::string& ip);
in6_addr ipv6_to_bits(const std::string& ip);
std::string ipv4_to_string(const in_addr& ip);
std::string ipv6_to_string(const in6_addr& ip);

std::size_t subnet_to_cidr_v4(const std::string& subnet_mask);
std::string ipv4_to_network(const std::string& ip, const std::string& subnet_mask);

} // namespace FLECS

#endif // F34AD8B9_5FE1_4B5D_A309_BBE14AD32A7A
