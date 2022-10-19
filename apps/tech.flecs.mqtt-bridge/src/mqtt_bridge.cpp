
#include "mqtt_bridge.h"

#include <cstdio>
#include <thread>

#include "util/cxx20/string.h"
#include "util/signal_handler/signal_handler.h"

namespace FLECS {

mqtt_bridge_t::mqtt_bridge_t() noexcept
    : _mqtt_client{std::make_unique<mqtt_client_t>()}
    , _flunder_client{std::make_unique<flunder_client_t>()}
    , _mqtt_thread{}
    , _flunder_thread{}
{}

mqtt_bridge_t::mqtt_bridge_t(mqtt_bridge_t&& other) noexcept
    : mqtt_bridge_t{}
{
    swap(*this, other);
}

mqtt_bridge_t& mqtt_bridge_t::operator=(mqtt_bridge_t&& other) noexcept
{
    swap(*this, other);
    return *this;
}

void swap(mqtt_bridge_t& lhs, mqtt_bridge_t& rhs) noexcept
{
    using std::swap;
    swap(lhs._mqtt_client, rhs._mqtt_client);
    swap(lhs._flunder_client, rhs._flunder_client);
    swap(lhs._mqtt_thread, rhs._mqtt_thread);
    swap(lhs._flunder_thread, rhs._flunder_thread);
}

auto mqtt_bridge_t::exec() //
    -> int
{
    _mqtt_thread = std::thread{&mqtt_bridge_t::mqtt_loop, this};
    _flunder_thread = std::thread{&mqtt_bridge_t::flunder_loop, this};

    _flunder_thread.join();
    _mqtt_thread.join();

    return 0;
}

template <typename Client, typename... Args>
void connect(std::string_view proto, Client* client, Args&&... args)
{
    while (!g_stop && (client->connect(std::forward<Args>(args)...)) != 0)
    {
        std::fprintf(stderr, "Could not connect %s - retrying in 10 seconds\n", proto.data());
        std::this_thread::sleep_for(std::chrono::seconds(10));
    }
    std::fprintf(stdout, "Connected to %s\n", proto.data());
}

auto mqtt_bridge_t::mqtt_loop() //
    -> void
{
    connect("mqtt", _mqtt_client.get(), MQTT_HOST, MQTT_PORT, MQTT_KEEPALIVE);

    _mqtt_client->receive_callback_set(mqtt_receive_callback, this);
    _mqtt_client->subscribe("#", 1);

    while (!g_stop)
    {
        std::this_thread::sleep_for(std::chrono::milliseconds(500));
    };
}

auto mqtt_bridge_t::flunder_loop() //
    -> void
{
    connect("flunder", _flunder_client.get(), FLUNDER_HOST, FLUNDER_PORT);

    _flunder_client->subscribe("**", flunder_receive_callback, this);

    while (!g_stop)
    {
        std::this_thread::sleep_for(std::chrono::milliseconds(500));
    };
}

auto mqtt_bridge_t::flunder_receive_callback(
    flunder_client_t* flunder_client, const flunder_variable_t* var, const void* userp) //
    -> void
{
    if (cxx20::starts_with(var->topic(), "/@"))
    {
        std::fprintf(stdout, "-- dropping message %s due to topic\n", var->topic().data());
        return;
    }
    if (var->encoding() == "application/mqtt-forwarded")
    {
        std::fprintf(
            stdout,
            "-- dropping message %s due to encoding %s\n",
            var->topic().data(),
            var->encoding().data());
    }

    decltype(auto) mqtt_bridge = const_cast<mqtt_bridge_t*>(static_cast<const mqtt_bridge_t*>(userp));

    flunder_client->add_mem_storage("flecs-mqtt-bridge", "**");

    int mid = 0;
    mqtt_bridge->mqtt_client().publish(
        var->topic().data(),
        &mid,
        static_cast<int>(var->len()),
        static_cast<const void*>(var->value().data()),
        1,
        false);
    mqtt_bridge->_pending_mids.insert(mid);
    std::fprintf(stdout, "++ forwarded flunder message for topic %s to mqtt (mid %d)\n", var->topic().data(), mid);
}

#if 0
void mqtt_bridge_t::expire_values()
{
    auto res = _flunder_client.get("/**");
    if (std::get<0>(res) != 0)
    {
        return;
    }
    decltype(auto) values = std::get<1>(res);
    for (const auto& value : values)
    {
        const auto now = std::chrono::system_clock().now().time_since_epoch().count();
    }
}
#endif // 0

void mqtt_bridge_t::mqtt_receive_callback(FLECS::mqtt_client_t* /*client*/, FLECS::mqtt_message_t* msg, void* userp)
{
    decltype(auto) mqtt_bridge = const_cast<mqtt_bridge_t*>(static_cast<const mqtt_bridge_t*>(userp));

    if (mqtt_bridge->_pending_mids.count(msg->id))
    {
        std::fprintf(stdout, "-- dropping mqtt message %s due to mid %d\n", msg->topic, msg->id);
        mqtt_bridge->_pending_mids.erase(msg->id);
        return;
    }

    mqtt_bridge->flunder_client().publish(
        static_cast<std::string_view>(msg->topic),
        static_cast<const void*>(msg->payload),
        static_cast<std::size_t>(msg->payloadlen),
        "application/mqtt-forwarded");

    std::fprintf(stdout, "++ forwarded mqtt message for topic %s to flunder\n", msg->topic);
}

} // namespace FLECS

int main()
{
    FLECS::signal_handler_init();

    auto bridge = FLECS::mqtt_bridge_t{};

    return bridge.exec();
}
