#ifndef D8B6ECD5_FBBE_47BF_A6F7_5D6CBDFCC738
#define D8B6ECD5_FBBE_47BF_A6F7_5D6CBDFCC738

#include <flunder/flunder_client.h>
#include <mqtt/mqtt_client.h>

#include <string>

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

    int loop();
    void expire_values();

    void connect_mqtt();
    void connect_flunder();

    void disconnect_mqtt();
    void disconnect_flunder();

    mqtt_client_t& mqtt_client() noexcept { return _mqtt_client; }
    flunder_client_t& flunder_client() noexcept { return _flunder_client; }

    bool mqtt_connected() const noexcept { return _mqtt_connected; }
    bool flunder_connected() const noexcept { return _flunder_connected; }

    void mqtt_connected(bool mqtt_connected) { _mqtt_connected = mqtt_connected; }
    void flunder_connected(bool flunder_connected) { _flunder_connected = flunder_connected; }

private:
    static void mqtt_receive_callback(FLECS::mqtt_client_t* client, FLECS::mqtt_message_t* msg, void* userp);
    static void mqtt_disconnect_callback(FLECS::mqtt_client_t* client, void* userp);

    friend void swap(mqtt_bridge_t& lhs, mqtt_bridge_t& rhs) noexcept;

    bool _mqtt_connected;
    bool _flunder_connected;

    mqtt_client_t _mqtt_client;
    flunder_client_t _flunder_client;
};

} // namespace FLECS

#endif // D8B6ECD5_FBBE_47BF_A6F7_5D6CBDFCC738
