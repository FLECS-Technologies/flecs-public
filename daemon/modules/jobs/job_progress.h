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

#include "core/flecs.h"
#include "job_id.h"
#include "job_status.h"
#include "util/json/json.h"

namespace FLECS {

class job_progress_t
{
public:
    job_progress_t()
        : _job_id{}
        , _status{}
        , _desc{}
        , _num_steps{}
        , _current_step{}
        , _result{}
        , _mutex{}
    {}

    struct current_step_t
    {
        std::string _desc;          /** current step description*/
        std::int16_t _num;          /** number of current step */
        std::string _unit;          /** unit of current step's operation (e.g. "B" when downloading)*/
        std::uint32_t _units_total; /** total units to process */
        std::uint32_t _units_done;  /** processed units so far*/
        std::uint32_t _rate;        /** processing rate in units per second*/

        current_step_t()
            : _desc{}
            , _num{}
            , _unit{}
            , _units_total{}
            , _units_done{}
            , _rate{}
        {}
    };

    job_progress_t(job_id_t job_id, std::string desc);

    auto job_id() const noexcept //
        -> job_id_t;

    auto status() const //
        -> job_status_e;
    auto desc() const noexcept //
        -> const std::string&;
    auto num_steps() const noexcept //
        -> std::int16_t;
    auto status(job_status_e status) noexcept //
        -> void;
    auto desc(std::string desc) noexcept //
        -> void;
    auto num_steps(std::int16_t num_steps) noexcept //
        -> void;

    auto current_step() const noexcept //
        -> const current_step_t&;

    auto next_step(std::string desc) //
        -> void;
    auto next_step(std::string desc, std::string unit, std::uint32_t units_total) //
        -> void;

    auto result() const noexcept //
        -> const result_t&;

    auto result(std::int32_t code) //
        -> void;
    auto result(std::int32_t code, std::string message) //
        -> void;

private:
    [[nodiscard]] auto lock() const //
        -> std::unique_lock<std::mutex>;

    friend auto to_json(json_t& j, const job_progress_t& progress) //
        -> void;

    job_id_t _job_id; /** unique job id */

    job_status_e _status;    /** @todo job status - replace by enum */
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
