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

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


