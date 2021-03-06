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

#ifndef DB4AFC45_7C99_43F7_BEE8_7B88F1AE7E9E
#define DB4AFC45_7C99_43F7_BEE8_7B88F1AE7E9E

#include <string>
#include <string_view>
#include <tuple>

namespace FLECS {

// generalized function return type: result code + error message
using result_t = std::tuple<int, std::string>;

} // namespace FLECS

#endif // DB4AFC45_7C99_43F7_BEE8_7B88F1AE7E9E
