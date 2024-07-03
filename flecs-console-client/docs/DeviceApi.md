# \DeviceApi

All URIs are relative to *https://console.flecs.tech*

Method | HTTP request | Description
------------- | ------------- | -------------
[**post_api_v2_device_license_activate**](DeviceApi.md#post_api_v2_device_license_activate) | **POST** /api/v2/device/license/activate | Activate license
[**post_api_v2_device_license_validate**](DeviceApi.md#post_api_v2_device_license_validate) | **POST** /api/v2/device/license/validate | Validate license



## post_api_v2_device_license_activate

> models::PostApiV2DeviceLicenseActivate200Response post_api_v2_device_license_activate(authorization, x_session_id, post_api_v2_device_license_activate_request)
Activate license

Activate device via user license or device serial number

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**authorization** | Option<**String**> | Provide only if activation via user license should be performed |  |
**x_session_id** | Option<[**SessionId**](.md)> | Leave out if no sessionId is known. |  |
**post_api_v2_device_license_activate_request** | Option<[**PostApiV2DeviceLicenseActivateRequest**](PostApiV2DeviceLicenseActivateRequest.md)> |  |  |

### Return type

[**models::PostApiV2DeviceLicenseActivate200Response**](post_api_v2_device_license_activate_200_response.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_api_v2_device_license_validate

> models::PostApiV2DeviceLicenseValidate200Response post_api_v2_device_license_validate(x_session_id)
Validate license

Validate the device license based on the session id

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**x_session_id** | **String** |  | [required] |

### Return type

[**models::PostApiV2DeviceLicenseValidate200Response**](post_api_v2_device_license_validate_200_response.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

