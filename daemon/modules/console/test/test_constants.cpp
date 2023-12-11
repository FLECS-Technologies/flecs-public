#include "test_constants.h"

const flecs::json_t auth_response_json = flecs::json_t::parse(R"-(
    {
        "status": 200,
        "statusText": "OK",
        "data": {
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
        }
    })-");

const flecs::json_t activate_response_json = flecs::json_t::parse(R"-(
    {
        "status": 200,
        "statusText": "OK",
        "data": {
            "sessionId": "{00000000-1111-2222-3333-444444444444}"
        }
    }
    )-");

const flecs::json_t validate_response_json = flecs::json_t::parse(R"-(
    {
        "status": 200,
        "statusText": "OK",
        "data": {
            "isValid": true
        }
    }
    )-");
