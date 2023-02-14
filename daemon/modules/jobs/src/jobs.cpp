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

#include "jobs.h"

#include "factory/factory.h"
#include "impl/jobs_impl.h"

namespace FLECS {

namespace {
register_module_t<module_jobs_t> _reg("jobs");
}

module_jobs_t::module_jobs_t()
    : _impl{new impl::module_jobs_t{}}
{}

module_jobs_t::~module_jobs_t() = default;

auto module_jobs_t::do_init() //
    -> void
{
    FLECS_V2_ROUTE("/jobs").methods("GET"_method)([this]() { return list_jobs({}); });
    FLECS_V2_ROUTE("/jobs/<uint>").methods("GET"_method)([this](std::uint32_t job_id) {
        return list_jobs(job_id_t{job_id});
    });

    _impl->do_init();
}

auto module_jobs_t::do_deinit() //
    -> void
{
    _impl->do_deinit();
}

auto module_jobs_t::append(job_t job, std::string desc) //
    -> job_id_t
{
    return _impl->do_append(std::move(job), std::move(desc));
}

auto module_jobs_t::list_jobs(job_id_t job_id) const //
    -> crow::response
{
    return _impl->do_list_jobs(std::move(job_id));
}

} // namespace FLECS
