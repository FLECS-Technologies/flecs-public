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

#include <cstdint>
#include <mutex>
#include <string>

#include "job_id.h"

namespace FLECS {

class job_progress_t
{
public:
    struct current_step_t
    {
        std::string _desc;          /** current step description*/
        std::int16_t _num;          /** number of current step */
        std::string _unit;          /** unit of current step's operation (e.g. "B" when downloading)*/
        std::uint32_t _units_total; /** total units to process */
        std::uint32_t _units_done;  /** processed units so far*/
        std::uint32_t _rate;        /** processing rate in units per second*/
    };

    struct result_t
    {
        std::int32_t code;
        std::string message;
    };

    explicit job_progress_t(job_id_t job_id);

    auto job_id() const noexcept //
        -> job_id_t;

    [[nodiscard]] auto lock() const //
        -> std::unique_lock<std::mutex>;

    auto status() const //
        -> std::uint32_t;
    auto desc() const noexcept //
        -> const std::string&;
    auto num_steps() const noexcept //
        -> std::int16_t;

    auto current_step() noexcept //
        -> current_step_t&;
    auto current_step() const noexcept //
        -> const current_step_t&;

    auto result() noexcept //
        -> result_t&;
    auto result() const noexcept //
        -> const result_t&;

private:
    job_id_t _job_id; /** unique job id */

    std::uint32_t _status;   /** @todo job status - replace by enum */
    std::string _desc;       /** job description (e.g. "Install app xyz (123) ")*/
    std::int16_t _num_steps; /** total number of steps  */

    current_step_t _current_step;
    result_t _result;

    mutable std::mutex _mutex;
};

auto operator<(const job_progress_t& lhs, const job_progress_t& rhs) //
    -> bool;
auto operator<=(const job_progress_t& lhs, const job_progress_t& rhs) //
    -> bool;
auto operator>(const job_progress_t& lhs, const job_progress_t& rhs) //
    -> bool;
auto operator>=(const job_progress_t& lhs, const job_progress_t& rhs) //
    -> bool;
auto operator==(const job_progress_t& lhs, const job_progress_t& rhs) //
    -> bool;
auto operator!=(const job_progress_t& lhs, const job_progress_t& rhs) //
    -> bool;

} // namespace  FLECS
