# Vsock

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**guest_cid** | **i32** | Guest Vsock CID | 
**uds_path** | **String** | Path to UNIX domain socket, used to proxy vsock connections. | 
**vsock_id** | Option<**String**> | This parameter has been deprecated and it will be removed in future Firecracker release. | [optional]
**dmb_size_mib** | Option<**i32**> | Device Memory Buffer size in MiB for confidential enclaves; defaults to 16 at attachment. Explicit sizes are rejected by vhost-user block and by KVM or direct-mode transports at attachment (InstanceStart or hotplug PUT). GET /vm/config reports supplied values before start and resolved sizes after attachment, including the default. KVM and direct mode omit it. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


