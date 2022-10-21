#ifndef D8B6ECD5_FBBE_47BF_A6F7_5D6CBDFCC738
#define D8B6ECD5_FBBE_47BF_A6F7_5D6CBDFCC738

#include <flunder/flunder_client.h>
#include <mqtt/mqtt_client.h>

#include <memory>
#include <set>
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
    ~mqtt_bridge_t() = default;

    auto exec() //
        -> int;

    auto mqtt_client() noexcept //
        -> mqtt_client_t&
    {
        return *_mqtt_client.get();
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

    static auto mqtt_receive_callback(mqtt_client_t*, mqtt_message_t*, void*) //
        -> void;
    static auto mqtt_disconnect_callback(mqtt_client_t*, void*) //
        -> void;

    std::unique_ptr<mqtt_client_t> _mqtt_client;
    std::unique_ptr<flunder_client_t> _flunder_client;
    std::set<int> _pending_mids;

    std::thread _mqtt_thread;
    std::thread _flunder_thread;
};

} // namespace FLECS

#endif // D8B6ECD5_FBBE_47BF_A6F7_5D6CBDFCC738
