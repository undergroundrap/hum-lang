# Session AC Windows Drive Locality Review

Date: 2026-07-11
Status: corrective implementation review packet; Session AC rejected pending
independent re-review

## Property And Threat Model

`fixed_local_v0` means that, under a trusted Windows kernel, trusted storage
drivers, and a non-deceptive hypervisor, the complete observable backing chain
contains no mapped, network, fabric, file-backed, virtual, removable, or
unknown layer. The property does not defend against a deceptive kernel,
storage driver, firmware, or hypervisor and is not a filesystem sandbox.

`GetDriveTypeW` reporting fixed media plus one ordinary `HarddiskVolumeN`
mapping is only preliminary evidence. It never produces `FixedLocal` alone.
The adapter also requires no type-2 storage dependency, a complete nonempty
volume extent list, a non-removable descriptor for every backing disk, bus type
exactly ATA, SATA, or NVMe, and identical drive/mapping observations after the
inspection.

The adapter receives only an already lexically validated drive root. It never
receives or opens the candidate path. Its public result may narrow an internal
Path or exact-grant fact; it grants no file authority.

## Isolation And Safe Public API

The local path dependency `crates/windows-drive-locality` is the entire FFI
boundary. The main `hum-lang` crate retains `#![forbid(unsafe_code)]`; the
adapter denies `unsafe_op_in_unsafe_fn`.

The safe public API remains:

- `DriveRoot::from_ascii_letter(u8) -> Option<DriveRoot>`;
- `classify(DriveRoot) -> DriveLocality`; and
- the closed `DriveLocality` enum.

Callers cannot supply handles, pointers, arbitrary paths, IOCTL values, raw
observations, storage descriptors, or foreign symbols. The complete injected
evidence model remains private to the adapter.

## Why Safe Rust Standard APIs Are Insufficient

`std::path` preserves and lexically decomposes a native Windows path but does
not expose drive type, DOS-device mapping, virtual-storage dependency, volume
extents, or physical storage bus. Fixed media is not equivalent to network-free
backing: VHD/VHDX files may live on remote volumes, iSCSI and NVMe-oF are fabric
transports, and virtual/file-backed/storage-spaces layers may still present a
fixed drive. Candidate metadata or candidate open would invert the required
safety ordering.

## Exact Foreign Signatures

The Windows-only `Kernel32` block declares:

```rust
fn GetDriveTypeW(lp_root_path_name: *const u16) -> u32;
fn QueryDosDeviceW(
    lp_device_name: *const u16,
    lp_target_path: *mut u16,
    ucch_max: u32,
) -> u32;
fn CreateFileW(
    lp_file_name: *const u16,
    desired_access: u32,
    share_mode: u32,
    security_attributes: *mut core::ffi::c_void,
    creation_disposition: u32,
    flags_and_attributes: u32,
    template_file: *mut core::ffi::c_void,
) -> *mut core::ffi::c_void;
fn DeviceIoControl(
    device: *mut core::ffi::c_void,
    control_code: u32,
    input: *mut core::ffi::c_void,
    input_size: u32,
    output: *mut core::ffi::c_void,
    output_size: u32,
    bytes_returned: *mut u32,
    overlapped: *mut core::ffi::c_void,
) -> i32;
fn CloseHandle(object: *mut core::ffi::c_void) -> i32;
```

The Windows-only `VirtDisk` block declares:

```rust
fn GetStorageDependencyInformation(
    object: *mut core::ffi::c_void,
    flags: u32,
    storage_dependency_info_size: u32,
    storage_dependency_info: *mut core::ffi::c_void,
    size_used: *mut u32,
) -> u32;
```

No other Windows symbol is declared.

## Authorized Query Chain

1. Query drive type and the bounded DOS-device mapping.
2. Continue only for `DRIVE_FIXED` plus one ordinary numbered hard-disk-volume
   mapping.
3. Synthesize the Win32 volume-device name from the validated letter and call
   `CreateFileW` with desired access zero, shared read/write/delete,
   `OPEN_EXISTING`, and no template or security descriptor.
4. Call `GetStorageDependencyInformation` with version 2 and host-volume
   direction. Any returned dependency entry, failure, or partial result blocks.
5. Call `DeviceIoControl` only with
   `IOCTL_VOLUME_GET_VOLUME_DISK_EXTENTS`. Require a bounded, complete,
   nonempty list and positive lengths.
6. For every unique extent disk number, synthesize the corresponding Win32
   physical-drive device name and open it with desired access zero.
7. Call `DeviceIoControl` only with `IOCTL_STORAGE_QUERY_PROPERTY`, using
   `StorageDeviceProperty` and `PropertyStandardQuery`. Require a complete
   descriptor, `RemovableMedia == false`, and bus exactly ATA, SATA, or NVMe.
8. Repeat drive type and DOS-device mapping and require byte-identical mapping
   plus identical drive type.
9. Close every successfully opened handle exactly once.

No candidate path participates in any step.

## Unsafe Inventory And Invariants

There are two unsafe foreign blocks and six unsafe call blocks:

1. `GetDriveTypeW`: an owned four-unit NUL-terminated drive-root buffer stays
   alive for the read-only call.
2. `QueryDosDeviceW`: an owned NUL-terminated drive device and exact 1024-unit
   writable UTF-16 buffer stay alive; the returned length is checked.
3. `CreateFileW`: the crate-synthesized NUL-terminated volume or physical-disk
   device name stays alive; desired access is zero; pointer parameters are
   null; the returned handle is checked against null and the invalid sentinel.
4. `DeviceIoControl`: the private handle is live; input/output slices match the
   passed lengths; the returned-byte pointer is valid; no overlapped operation
   is used.
5. `GetStorageDependencyInformation`: the private volume handle is live; the
   64-KiB aligned buffer begins with the type-2 tag; its exact bound and the
   size-used result are checked.
6. `CloseHandle`: each successful `CreateFileW` result is owned by one private
   handle value and closed exactly once. Every normal classification path
   observes the close result, and any close failure blocks `FixedLocal`. Drop
   retains only a best-effort fallback for an already-failing or unwinding path.

No raw pointer, handle, buffer, observation, or IOCTL escapes the crate.

## Bounds, Layout, And Failure Mapping

- DOS mapping: 1024 UTF-16 units; nonzero, in-bounds, one nonempty terminated
  mapping only.
- Dependency information: aligned 64 KiB; version 2; success; size used from 8
  bytes through the bound; zero entries only.
- Volume extents: aligned 64 KiB; at most 1024 complete extents; ABI offsets and
  sizes are pinned by tests; every length is positive and every offset is
  nonnegative.
- Device descriptor: aligned 4096 bytes; version and reported size must cover
  the 36-byte fixed header and remain within returned bytes.
- Device names are synthesized only from the validated ASCII drive letter or a
  returned `u32` disk number.

| Evidence | Result |
| --- | --- |
| Stable fixed mapping, no dependency, complete extents, all direct non-removable ATA/SATA/NVMe | `FixedLocal` |
| Remote/MUP | `Remote` |
| substituted/DOS-device indirection | `Substituted` |
| removable drive class | `Removable` |
| non-Windows host | `Unsupported` |
| Any dependency, disallowed/removable bus, incomplete topology, API/buffer failure, topology change, or unknown/future value | `Unknown` |

Every non-`FixedLocal` result remains `locality_unclassified` in Hum.

## Deterministic Test Matrix

Private injected evidence covers:

- direct ATA, SATA, and NVMe positives;
- VHD, VHDX, ISO, and UNC-hosted-VHD dependency observations;
- iSCSI, NVMe-oF, Virtual, FileBackedVirtual, Spaces, Fibre, RAID, SCSI, and
  SAS buses;
- removable backing disks;
- multiple extents, repeated disk extents, and multiple allowed disks;
- empty, zero-length, negative-offset, missing-disk, and extra-disk topology;
- direct dependency, extent, and storage-descriptor decoder tests for API
  failure, truncation, undersized/oversized results, wrong version/count, and
  inconsistent lengths;
- injected close failure plus normalized API/partial state at every observation
  stage;
- changed mapping and changed drive type at the stable recheck;
- remote, substituted, removable, malformed, and unknown preliminary facts;
- future unknown bus values;
- exact Windows ABI layout and DOS-mapping buffer bounds; and
- non-Windows `Unsupported` behavior with foreign code excluded by
  `cfg(windows)`.

The hosted Windows repository-drive smoke requires only a closed Windows
classification. It may return `Unknown` when the host exposes a virtual,
fabric, inaccessible, or otherwise incomplete chain; it must never force a
false `FixedLocal` result.

## Forbidden Operations And TCB Impact

The adapter contains no candidate metadata, candidate open, canonicalization,
directory enumeration, content access, WMI, COM, SetupDi, registry access,
process creation, environment read, network call, dynamic loading, arbitrary
IOCTL, vendor/model heuristic, broad binding crate, or arbitrary symbol lookup.
It uses no third-party dependency.

The TCB increase is limited to the private evidence classifier, six exact
foreign declarations, two foreign blocks, six unsafe calls, bounded decoders,
synthesized device names, and private query-only handle lifetime. The semantic
claim explicitly trusts the Windows kernel, storage-driver stack, and
non-deceptive hypervisor.

## Rollback

Rollback removes the local path dependency and `fixed_local_v0` narrowing,
returning all Session AB Path and grant values to `locality_unclassified`.
There is no source, schema, capability, stored-grant, or Core migration because
Session AC adds no public language surface.
