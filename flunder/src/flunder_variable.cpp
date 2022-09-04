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

flunder_variable_t::flunder_variable_t(
    std::string_view key, std::string_view value, std::string_view encoding, std::string_view timestamp)
    : _key{new char[key.length() + 1]}
    , _value{new char[value.length() + 1 + 1]}
    , _encoding{new char[encoding.length() + 1]}
    , _timestamp{new char[timestamp.length() + 1]}
{
    std::strcpy(_key, key.data());
    std::strcpy(_value, value.data());
    std::strcpy(_encoding, encoding.data());
    std::strcpy(_timestamp, timestamp.data());
}

flunder_variable_t::flunder_variable_t(const flunder_variable_t& other)
    : flunder_variable_t{other._key, other._value, other._encoding, other._timestamp}
{}

flunder_variable_t::flunder_variable_t(flunder_variable_t&& other)
    : flunder_variable_t{}
{
    swap(*this, other);
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

void swap(flunder_variable_t& lhs, flunder_variable_t& rhs)
{
    using std::swap;
    swap(lhs._key, rhs._key);
    swap(lhs._value, rhs._value);
    swap(lhs._encoding, rhs._encoding);
    swap(lhs._timestamp, rhs._timestamp);
}

flunder_variable_t* flunder_variable_new(
    const char* key, const char* value, const char* encoding, const char* timestamp)
{
    return new flunder_variable_t{key, value, encoding, timestamp};
}

flunder_variable_t* flunder_variable_clone(const flunder_variable_t* other)
{
    return new flunder_variable_t{*other};
}

flunder_variable_t* flunder_variable_move(flunder_variable_t* other)
{
    return new flunder_variable_t{std::move(*other)};
}

void flunder_variable_destroy(flunder_variable_t* var)
{
    delete var;
}

} // namespace FLECS
