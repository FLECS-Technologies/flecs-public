#ifndef D8B6ECD5_FBBE_47BF_A6F7_5D6CBDFCC738
#define D8B6ECD5_FBBE_47BF_A6F7_5D6CBDFCC738

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

    static auto flunder_receive_callback(flunder_client_t*, const flunder_variable_t*, const void*) //
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

#endif // D8B6ECD5_FBBE_47BF_A6F7_5D6CBDFCC738
