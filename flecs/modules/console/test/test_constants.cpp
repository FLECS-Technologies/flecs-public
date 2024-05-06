#include "test_constants.h"

const flecs::json_t auth_response_json = flecs::json_t::parse(R"-(
    {
        "statusCode": 200,
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
        "statusCode": 200,
        "statusText": "OK",
        "data": {
            "sessionId": {
                "id": "{00000000-1111-2222-3333-444444444444}",
                "timestamp": 1235
            },
            "licenseKey": "some-license-key"
        }
    }
)-");

const flecs::json_t create_token_response_json = flecs::json_t::parse(R"-(
    {
        "statusCode": 200,
        "statusText": "OK",
        "data": {
            "token": {
                "username": "username",
                "password": "password"
            }
        }
    }
)-");

const flecs::json_t error_response_json = flecs::json_t::parse(R"-(
    {
        "statusCode": 500,
        "statusText": "Internal Server Error",
        "reason": "Something went wrong..."
    }
)-");

const flecs::json_t validate_response_json = flecs::json_t::parse(R"-(
    {
        "statusCode": 200,
        "statusText": "OK",
        "data": {
            "isValid": true
        }
    }
)-");
