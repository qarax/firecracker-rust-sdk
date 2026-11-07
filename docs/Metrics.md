# Metrics

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**metrics_path** | **String** | Path to the named pipe or file where the JSON-formatted metrics are flushed. | 
**emit_id** | Option<**bool**> | Whether to emit the microVM instance id (the value passed to the jailer \"--id\") as a top-level \"id\" field on each metrics line. | [optional][default to false]
**properties** | Option<**std::collections::HashMap<String, String>**> | Operator-defined key-value pairs emitted under a top-level \"properties\" field on each metrics line. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


