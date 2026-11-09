# NetworkInterface

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**guest_mac** | Option<**String**> |  | [optional]
**host_dev_name** | **String** | Host level path for the guest network interface | 
**iface_id** | **String** |  | 
**mtu** | Option<**i32**> | MTU to advertise to the guest via VIRTIO_NET_F_MTU. When set, a compatible guest driver will configure the interface with this MTU. When absent, VIRTIO_NET_F_MTU is not advertised and the guest uses its default MTU. | [optional]
**rx_rate_limiter** | Option<[**models::RateLimiter**](RateLimiter.md)> |  | [optional]
**tx_rate_limiter** | Option<[**models::RateLimiter**](RateLimiter.md)> |  | [optional]
**dmb_size_mib** | Option<**i32**> | Device Memory Buffer size in MiB for confidential enclaves; defaults to 16 at attachment. Explicit sizes are rejected by vhost-user block and by KVM or direct-mode transports at attachment (InstanceStart or hotplug PUT). GET /vm/config reports supplied values before start and resolved sizes after attachment, including the default. KVM and direct mode omit it. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


