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

#ifndef ABC7B599_2593_43CC_856A_CDB2D9CF3D01
#define ABC7B599_2593_43CC_856A_CDB2D9CF3D01

#ifdef __cplusplus

#include <string_view>

namespace FLECS {
extern "C" {
#endif // __cplusplus

struct flunder_variable_t
{
#ifdef __cplusplus
    flunder_variable_t();
    flunder_variable_t(
        std::string_view key, std::string_view value, std::string_view encoding, std::string_view timestamp);
    flunder_variable_t(const flunder_variable_t& other);
    flunder_variable_t(flunder_variable_t&& other);
    flunder_variable_t operator=(flunder_variable_t other);
    ~flunder_variable_t();

    friend void swap(flunder_variable_t& lhs, flunder_variable_t& rhs);
#endif //__cplusplus
    char* _key;
    char* _value;
    char* _encoding;
    char* _timestamp;
};

struct flunder_variable_t* flunder_variable_new(
    const char* key, const char* value, const char* encoding, const char* timestamp);
struct flunder_variable_t* flunder_variable_clone(const struct flunder_variable_t* other);
struct flunder_variable_t* flunder_variable_move(struct flunder_variable_t* other);
void flunder_variable_destroy(struct flunder_variable_t* var);

#ifdef __cplusplus
} // extern "C"
} // namespace FLECS
#endif // __cplusplus

#endif // ABC7B599_2593_43CC_856A_CDB2D9CF3D01
