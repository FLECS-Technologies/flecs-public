#include <pybind11/functional.h>
#include <pybind11/pybind11.h>

#include "mqtt/mqtt_client.h"

namespace py = pybind11;

PYBIND11_MODULE(mqtt_client, m)
{
    using receive_callback_set_t = int (FLECS::mqtt_client_t::*)(FLECS::mqtt_client_t::mqtt_receive_callback_t);
    py::class_<FLECS::mqtt_client_t>(m, "mqtt_client")
        .def(py::init())
        .def("connect", (int(FLECS::mqtt_client_t::*)())(&FLECS::mqtt_client_t::connect))
        .def("connect", (int(FLECS::mqtt_client_t::*)(const char*, int, int))(&FLECS::mqtt_client_t::connect))
        .def("reconnect", &FLECS::mqtt_client_t::reconnect)
        .def("disconnect", &FLECS::mqtt_client_t::disconnect)
        .def("is_connected", &FLECS::mqtt_client_t::is_connected)
        .def("subscribe", &FLECS::mqtt_client_t::subscribe)
        .def("unsubscribe", &FLECS::mqtt_client_t::unsubscribe)
        .def(
            "publish",
            (int(FLECS::mqtt_client_t::*)(const char*, int, const void*, int, bool)
                 const)(&FLECS::mqtt_client_t::publish))
        .def(
            "publish",
            (int(FLECS::mqtt_client_t::*)(const char*, int*, int, const void*, int, bool)
                 const)(&FLECS::mqtt_client_t::publish))
        .def("receive_callback_set", (receive_callback_set_t)(&FLECS::mqtt_client_t::receive_callback_set))
        .def("receive_callback_clear", &FLECS::mqtt_client_t::receive_callback_clear);
}
