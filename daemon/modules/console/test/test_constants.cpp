#include "test_constants.h"

const flecs::json_t auth_json = flecs::json_t::parse(R"-(
    {
        "user": {
            "ID": 123,
            "user_email": "user@flecs.tech",
            "user_login": "user",
            "display_name": "Some FLECS user"
        },
        "jwt": {
            "token": "eyJ0eXAiO...",
            "token_expires": 1641034800
        },
        "feature_flags": {
            "isVendor": true,
            "isWhitelabeled": false
        }
    })-");
