# \DefaultApi

All URIs are relative to *https://console.flecs.tech*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_api_v2_manifests_app_version**](DefaultApi.md#get_api_v2_manifests_app_version) | **GET** /api/v2/manifests/{app}/{version} | Download App manifest
[**post_api_v2_auth_login**](DefaultApi.md#post_api_v2_auth_login) | **POST** /api/v2/auth/login | Log in
[**post_api_v2_auth_validate**](DefaultApi.md#post_api_v2_auth_validate) | **POST** /api/v2/auth/validate | Create api validate
[**post_api_v2_tokens**](DefaultApi.md#post_api_v2_tokens) | **POST** /api/v2/tokens | Acquire download token for App



## get_api_v2_manifests_app_version

> models::GetApiV2ManifestsAppVersion200Response get_api_v2_manifests_app_version(x_session_id, app, version)
Download App manifest

Get the app manifest for a given app and version

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**x_session_id** | **String** |  | [required] |
**app** | **String** |  | [required] |
**version** | **String** |  | [required] |

### Return type

[**models::GetApiV2ManifestsAppVersion200Response**](get_api_v2_manifests_app_version_200_response.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_api_v2_auth_login

> models::Apiv2AuthLoginPostResponse post_api_v2_auth_login(apiv2_auth_login_post_request)
Log in

Login to console as user with password

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**apiv2_auth_login_post_request** | Option<[**Apiv2AuthLoginPostRequest**](Apiv2AuthLoginPostRequest.md)> |  |  |

### Return type

[**models::Apiv2AuthLoginPostResponse**](APIV2AuthLoginPostResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_api_v2_auth_validate

> models::Apiv2AuthValidatePostResponse post_api_v2_auth_validate(apiv2_auth_validate_post_request)
Create api validate

Validate authentication token

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**apiv2_auth_validate_post_request** | Option<[**Apiv2AuthValidatePostRequest**](Apiv2AuthValidatePostRequest.md)> |  |  |

### Return type

[**models::Apiv2AuthValidatePostResponse**](APIV2AuthValidatePostResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_api_v2_tokens

> models::PostApiV2Tokens200Response post_api_v2_tokens(x_session_id, post_api_v2_tokens_request)
Acquire download token for App

Create a download token for a given app and version

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**x_session_id** | **String** |  | [required] |
**post_api_v2_tokens_request** | Option<[**PostApiV2TokensRequest**](PostApiV2TokensRequest.md)> |  |  |

### Return type

[**models::PostApiV2Tokens200Response**](post_api_v2_tokens_200_response.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

