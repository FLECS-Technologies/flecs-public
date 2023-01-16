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

#pragma once

#include <memory>

#include "module_base/module.h"

namespace FLECS {

namespace impl {
class module_manifests_t;
} // namespace impl

class module_manifests_t FLECS_FINAL_UNLESS_TESTED : public module_t
{
    friend class module_factory_t;

public:
    ~module_manifests_t() override;

protected:
    module_manifests_t();

    auto do_init() //
        -> void override;
    auto do_deinit() //
        -> void override;

    std::unique_ptr<impl::module_manifests_t> _impl;
};

} // namespace FLECS
