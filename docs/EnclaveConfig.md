# EnclaveConfig

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**cpu_ids** | Option<**Vec<i32>**> | Specific CPU IDs from the Nitro Enclaves CPU pool. When omitted, the kernel selects CPUs from the dynamic pool. When present, the number of IDs must equal the machine vCPU count. | [optional]
**debug_mode** | Option<**bool**> | Whether to enable enclave debug mode, which exposes guest serial output through the configured serial output sink. | [optional][default to false]
**direct_mode** | Option<**bool**> | Whether to run in direct, non-confidential mode. The default is confidential mode, which uses per-device data memory buffers (DMBs) sized by each device's `dmb_size_mib`. | [optional][default to false]
**cpu_overcommit** | Option<**bool**> | Whether to use host-scheduled floating vCPUs from the parent's overcommit pool instead of dedicated pinned cores from the Nitro Enclaves CPU pool. | [optional][default to false]
**cpu_accounting** | Option<**bool**> | Whether to attribute each enclave vCPU's guest time to a parent-side thread. Spawns an additional host thread per vCPU, mirroring the vCPU's CPU time, therefore making it host-visible. | [optional][default to false]
**hotplug_mode** | Option<**bool**> | Whether PCIe devices can be attached after the enclave starts. | [optional][default to false]
**numa_nodes** | Option<[**Vec<models::EnclaveNumaNode>**](EnclaveNumaNode.md)> | Per-parent-NUMA-node placement of vCPUs and memory. When omitted, all vCPUs and memory are placed on parent NUMA node 0. The vCPU and memory totals must equal the machine configuration, and parent NUMA node IDs must be unique. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


