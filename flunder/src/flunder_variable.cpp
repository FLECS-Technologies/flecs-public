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

#include "flunder_variable.h"

#include <cstring>
#include <memory>

namespace FLECS {

flunder_variable_t::flunder_variable_t()
    : _key{}
    , _value{}
    , _encoding{}
    , _timestamp{}
{}

flunder_variable_t::flunder_variable_t(const char* key, const char* value, const char* encoding, const char* timestamp)
{
    auto cp_key = new char[std::strlen(key) + 1]();
    auto cp_value = new char[std::strlen(value) + 1]();
    auto cp_encoding = new char[std::strlen(encoding) + 1]();
    auto cp_timestamp = new char[std::strlen(timestamp) + 1]();

    std::strcpy(cp_key, key);
    std::strcpy(cp_value, value);
    std::strcpy(cp_encoding, encoding);
    std::strcpy(cp_timestamp, timestamp);

    _key = cp_key;
    _value = cp_value;
    _encoding = cp_encoding;
    _timestamp = cp_timestamp;
}

flunder_variable_t::flunder_variable_t(const flunder_variable_t& other)
    : flunder_variable_t{other._key, other._value, other._encoding, other._timestamp}
{}

flunder_variable_t::flunder_variable_t(flunder_variable_t&& other)
    : _key{other._key}
    , _value{other._value}
    , _encoding{other._encoding}
    , _timestamp{other._timestamp}
{
    other._key = nullptr;
    other._value = nullptr;
    other._encoding = nullptr;
    other._timestamp = nullptr;
}

flunder_variable_t flunder_variable_t::operator=(flunder_variable_t other)
{
    swap(*this, other);
    return *this;
}

flunder_variable_t::~flunder_variable_t()
{
    delete[] _key;
    delete[] _value;
    delete[] _encoding;
    delete[] _timestamp;
}

flunder_variable_t* flunder_variable_new(
    const char* key, const char* value, const char* encoding, const char* timestamp)
{
    return new flunder_variable_t{key, value, encoding, timestamp};
}

flunder_variable_t* flunder_variable_clone(flunder_variable_t* other)
{
    return new flunder_variable_t{*other};
}

flunder_variable_t* flunder_variable_move(flunder_variable_t* other)
{
    return new flunder_variable_t{std::move(*other)};
}

void swap(flunder_variable_t& lhs, flunder_variable_t& rhs)
{
    using std::swap;
    swap(lhs._key, rhs._key);
    swap(lhs._value, rhs._value);
    swap(lhs._encoding, rhs._encoding);
    swap(lhs._timestamp, rhs._timestamp);
}

void flunder_variable_destroy(flunder_variable_t* var)
{
    delete var;
}

} // namespace FLECS
