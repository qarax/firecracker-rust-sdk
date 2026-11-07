# SnapshotCreateParams

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**mem_file_path** | **String** | Path to the file that will contain the guest memory. | 
**snapshot_path** | **String** | Path to the file that will contain the microVM state. | 
**snapshot_type** | Option<**SnapshotType**> | Type of snapshot to create. It is optional and by default, a full snapshot is created. (enum: Full, Diff) | [optional]
**sync_snapshot_files** | Option<**bool**> | Whether the snapshot state and guest memory files are synced to disk (fsync) before the request returns. Optional, defaults to true, guaranteeing durability across host crashes. Setting it to false returns without waiting for the writeback, making snapshot creation faster at the cost of durability across a host crash; the data stays in the host page cache, so same-host reads still see the full contents. Block device backing files are always fsync'd regardless of this setting. | [optional][default to true]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


