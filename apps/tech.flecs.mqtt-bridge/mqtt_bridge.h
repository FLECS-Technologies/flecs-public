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

#include <flunder/flunder_client.h>
#include <mosquitto.h>

#include <memory>
#include <thread>

namespace FLECS {

class mqtt_bridge_t
{
public:
    mqtt_bridge_t() noexcept;
    mqtt_bridge_t(const mqtt_bridge_t&) = delete;
    mqtt_bridge_t(mqtt_bridge_t&& other) noexcept;
    mqtt_bridge_t& operator=(const mqtt_bridge_t&) = delete;
    mqtt_bridge_t& operator=(mqtt_bridge_t&& other) noexcept;
    ~mqtt_bridge_t();

    auto exec() //
        -> int;

    auto mosq() noexcept //
        -> mosquitto*
    {
        return _mosq;
    }

    auto mqtt_connected() const noexcept //
        -> bool
    {
        return _mqtt_connected;
    }

    auto flunder_client() noexcept //
        -> flunder_client_t&
    {
        return *_flunder_client.get();
    }

private:
    friend void swap(mqtt_bridge_t& lhs, mqtt_bridge_t& rhs) noexcept;

    auto mqtt_loop() //
        -> void;
    auto flunder_loop() //
        -> void;

    static auto flunder_receive_callback(
        flunder_client_t*, const flunder_variable_t*, const void*) //
        -> void;

    static auto mosquitto_receive_callback(mosquitto*, void*, const mosquitto_message*) //
        -> void;
    static auto mosquitto_connect_callback(mosquitto*, void*, int) //
        -> void;
    static auto mosquitto_disconnect_callback(mosquitto*, void*, int) //
        -> void;

    mosquitto* _mosq;
    bool _mqtt_connected;
    std::unique_ptr<flunder_client_t> _flunder_client;

    std::thread _mqtt_thread;
    std::thread _flunder_thread;
};

} // namespace FLECS
