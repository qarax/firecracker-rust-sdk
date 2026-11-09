# Drive

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**drive_id** | **String** |  | 
**partuuid** | Option<**String**> | Represents the unique id of the boot partition of this device. It is optional and it will be taken into account only if the is_root_device field is true. | [optional]
**is_root_device** | **bool** |  | 
**cache_type** | Option<**CacheType**> | Represents the caching strategy for the block device. (enum: Unsafe, Writeback) | [optional][default to Unsafe]
**is_read_only** | Option<**bool**> | Is block read only. This field is required for virtio-block config and should be omitted for vhost-user-block configuration. | [optional]
**discard** | Option<**bool**> | Enables virtio-blk discard support. When enabled, the guest can issue discard/TRIM requests for this drive. Only supported for writable virtio-block devices using the Sync IO engine. This field is optional for virtio-block config and should be omitted for vhost-user-block configuration. | [optional][default to false]
**path_on_host** | Option<**String**> | Host level path for the guest drive. This field is required for virtio-block config and should be omitted for vhost-user-block configuration. | [optional]
**rate_limiter** | Option<[**models::RateLimiter**](RateLimiter.md)> |  | [optional]
**io_engine** | Option<**IoEngine**> | Type of the IO engine used by the device. \"Async\" is supported on host kernels newer than 5.10.51. This field is optional for virtio-block config and should be omitted for vhost-user-block configuration. (enum: Sync, Async) | [optional][default to Sync]
**blk_size** | Option<**i32**> | Logical block size advertised to the guest via VIRTIO_BLK_F_BLK_SIZE. This field is optional for virtio-block config and should be omitted for vhost-user-block configuration. | [optional]
**topology** | Option<[**models::BlockTopology**](BlockTopology.md)> |  | [optional]
**socket** | Option<**String**> | Path to the socket of vhost-user-block backend. This field is required for vhost-user-block config should be omitted for virtio-block configuration. | [optional]
**dmb_size_mib** | Option<**i32**> | Device Memory Buffer size in MiB for confidential enclaves; defaults to 16 at attachment. Explicit sizes are rejected by vhost-user block and by KVM or direct-mode transports at attachment (InstanceStart or hotplug PUT). GET /vm/config reports supplied values before start and resolved sizes after attachment, including the default. KVM and direct mode omit it. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


