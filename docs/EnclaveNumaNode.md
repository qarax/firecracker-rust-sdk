# EnclaveNumaNode

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**parent_numa** | **i32** | Parent NUMA node ID. | 
**vcpu_count** | **i32** | Number of enclave vCPUs to place on this parent NUMA node. The total across all nodes must equal the machine vCPU count. | 
**mem_size_mib** | **i32** | Enclave memory in MiB to place on this parent NUMA node. Every entry except the last, after sorting by parent_numa, must be a multiple of 128 MiB. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


