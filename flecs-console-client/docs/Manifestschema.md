# Manifestschema

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**_schema_version** | Option<**String**> | Version of the implemented FLECS App Manifest schema | [optional]
**app** | **String** | Unique App identifier in reverse domain name notation | 
**version** | **String** | App version, naturally sortable | 
**revision** | Option<**String**> | App manifest revision. Increment if Manifest is changed within the same App version | [optional]
**image** | **String** | Docker image for the App | 
**multi_instance** | Option<**bool**> | 'true' if App can be instantiated more than once (requires no editor, no ports) | [optional]
**editor** | Option<**String**> | Port of App's web-based ui | [optional]
**args** | Option<**Vec<String>**> | Command line arguments passed to App entrypoint | [optional]
**capabilities** | Option<[**Vec<serde_json::Value>**](serde_json::Value.md)> | Permissions required for the App to function correctly | [optional]
**conffiles** | Option<**Vec<String>**> | Configuration files used by the App | [optional]
**devices** | Option<**Vec<String>**> | Devices passed through to the App instances | [optional]
**env** | Option<**Vec<String>**> | Environment variables for the App instances | [optional]
**interactive** | Option<**bool**> | DEPRECATED: true if App requires allocation of an interactive shell | [optional]
**ports** | Option<**Vec<String>**> | Port mappings for the App's instances (not allowed for multiInstance Apps) | [optional]
**volumes** | Option<**Vec<String>**> | Virtual volumes and bind mounts for an App's instances | [optional]
**labels** | Option<**Vec<String>**> | Labels for the App instances | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


