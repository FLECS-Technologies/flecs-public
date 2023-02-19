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

#include "module_base/module.h"

namespace FLECS {

namespace impl {
class module_flecsport_t;
} // namespace impl

class module_flecsport_t FLECS_FINAL_UNLESS_TESTED : public module_t
{
public:
    ~module_flecsport_t() override;

protected:
    friend class module_factory_t;

    module_flecsport_t();

    auto do_init() //
        -> void override
    {}

    auto do_deinit() //
        -> void override
    {}

    std::unique_ptr<impl::module_flecsport_t> _impl;
};

} // namespace FLECS
