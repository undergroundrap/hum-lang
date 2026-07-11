#![deny(unsafe_op_in_unsafe_fn)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DriveRoot {
    letter: u16,
    root: [u16; 4],
    device: [u16; 3],
}

impl DriveRoot {
    pub fn from_ascii_letter(letter: u8) -> Option<Self> {
        letter.is_ascii_alphabetic().then(|| {
            let letter = u16::from(letter.to_ascii_uppercase());
            Self {
                letter,
                root: [letter, u16::from(b':'), u16::from(b'\\'), 0],
                device: [letter, u16::from(b':'), 0],
            }
        })
    }

    #[cfg(windows)]
    fn volume_device(self) -> [u16; 7] {
        [
            u16::from(b'\\'),
            u16::from(b'\\'),
            u16::from(b'.'),
            u16::from(b'\\'),
            self.letter,
            u16::from(b':'),
            0,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriveLocality {
    FixedLocal,
    Remote,
    Substituted,
    Removable,
    Unsupported,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg(any(windows, test))]
enum DriveTypeObservation {
    Unknown,
    MissingRoot,
    Removable,
    Fixed,
    Remote,
    Optical,
    RamDisk,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg(any(windows, test))]
enum QueryState<T> {
    Complete(T),
    ApiFailure,
    Partial,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg(any(windows, test))]
struct PreliminaryObservation {
    drive_type: DriveTypeObservation,
    mapping: QueryState<Vec<u16>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg(any(windows, test))]
enum DependencyObservation {
    None,
    #[cfg(windows)]
    Present,
    #[cfg(test)]
    Vhd,
    #[cfg(test)]
    Vhdx,
    #[cfg(test)]
    Iso,
    #[cfg(test)]
    UncHostedVhd,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg(any(windows, test))]
struct ExtentObservation {
    disk_number: u32,
    starting_offset: i64,
    extent_length: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg(any(windows, test))]
struct DiskObservation {
    disk_number: u32,
    removable: bool,
    bus_type: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg(any(windows, test))]
struct InspectionEvidence {
    before: PreliminaryObservation,
    dependency: QueryState<DependencyObservation>,
    extents: QueryState<Vec<ExtentObservation>>,
    disks: QueryState<Vec<DiskObservation>>,
    closes: QueryState<()>,
    after: PreliminaryObservation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg(any(windows, test))]
enum PreliminaryClass {
    Candidate,
    Closed(DriveLocality),
}

#[cfg(any(windows, test))]
fn classify_preliminary(observation: &PreliminaryObservation) -> PreliminaryClass {
    let target = match &observation.mapping {
        QueryState::Complete(target) if !target.is_empty() => target.as_slice(),
        QueryState::Complete(_) | QueryState::ApiFailure | QueryState::Partial => {
            return PreliminaryClass::Closed(DriveLocality::Unknown);
        }
    };

    if starts_ascii_case_insensitive(target, r"\Device\Mup")
        || starts_ascii_case_insensitive(target, r"\Device\LanmanRedirector")
    {
        return PreliminaryClass::Closed(DriveLocality::Remote);
    }
    if starts_ascii_case_insensitive(target, r"\??\")
        || starts_ascii_case_insensitive(target, r"\DosDevices\")
    {
        return PreliminaryClass::Closed(DriveLocality::Substituted);
    }

    match observation.drive_type {
        DriveTypeObservation::Remote => PreliminaryClass::Closed(DriveLocality::Remote),
        DriveTypeObservation::Removable
        | DriveTypeObservation::Optical
        | DriveTypeObservation::RamDisk => PreliminaryClass::Closed(DriveLocality::Removable),
        DriveTypeObservation::Fixed if is_ordinary_fixed_target(target) => {
            PreliminaryClass::Candidate
        }
        DriveTypeObservation::Unknown
        | DriveTypeObservation::MissingRoot
        | DriveTypeObservation::Fixed
        | DriveTypeObservation::Other => PreliminaryClass::Closed(DriveLocality::Unknown),
    }
}

#[cfg(any(windows, test))]
fn classify_evidence(evidence: &InspectionEvidence) -> DriveLocality {
    let PreliminaryClass::Candidate = classify_preliminary(&evidence.before) else {
        let PreliminaryClass::Closed(result) = classify_preliminary(&evidence.before) else {
            unreachable!();
        };
        return result;
    };

    if evidence.before != evidence.after
        || classify_preliminary(&evidence.after) != PreliminaryClass::Candidate
        || evidence.dependency != QueryState::Complete(DependencyObservation::None)
        || evidence.closes != QueryState::Complete(())
    {
        return DriveLocality::Unknown;
    }

    let QueryState::Complete(extents) = &evidence.extents else {
        return DriveLocality::Unknown;
    };
    let QueryState::Complete(disks) = &evidence.disks else {
        return DriveLocality::Unknown;
    };
    if extents.is_empty()
        || extents
            .iter()
            .any(|extent| extent.starting_offset < 0 || extent.extent_length <= 0)
    {
        return DriveLocality::Unknown;
    }

    let mut required_disks = extents
        .iter()
        .map(|extent| extent.disk_number)
        .collect::<Vec<_>>();
    required_disks.sort_unstable();
    required_disks.dedup();
    if disks.len() != required_disks.len() {
        return DriveLocality::Unknown;
    }

    for disk_number in required_disks {
        let Some(disk) = disks.iter().find(|disk| disk.disk_number == disk_number) else {
            return DriveLocality::Unknown;
        };
        if disk.removable || !matches!(disk.bus_type, BUS_TYPE_ATA | BUS_TYPE_SATA | BUS_TYPE_NVME)
        {
            return DriveLocality::Unknown;
        }
    }

    DriveLocality::FixedLocal
}

#[cfg(not(windows))]
pub fn classify(_root: DriveRoot) -> DriveLocality {
    DriveLocality::Unsupported
}

#[cfg(windows)]
pub fn classify(root: DriveRoot) -> DriveLocality {
    let before = query_preliminary(root);
    if let PreliminaryClass::Closed(result) = classify_preliminary(&before) {
        return result;
    }

    let Some(volume) = open_device(&root.volume_device()) else {
        return DriveLocality::Unknown;
    };
    let dependency = query_dependencies(&volume);
    if dependency != QueryState::Complete(DependencyObservation::None) {
        let _already_unknown = volume.close();
        return DriveLocality::Unknown;
    }
    let extents = query_extents(&volume);
    let (disks, disk_closes) = match &extents {
        QueryState::Complete(extents) => query_backing_disks(extents),
        QueryState::ApiFailure => (QueryState::ApiFailure, true),
        QueryState::Partial => (QueryState::Partial, true),
    };
    let volume_closed = volume.close();
    let after = query_preliminary(root);

    classify_evidence(&InspectionEvidence {
        before,
        dependency,
        extents,
        disks,
        closes: if volume_closed && disk_closes {
            QueryState::Complete(())
        } else {
            QueryState::ApiFailure
        },
        after,
    })
}

#[cfg(any(windows, test))]
fn is_ordinary_fixed_target(target: &[u16]) -> bool {
    const PREFIX: &str = r"\Device\HarddiskVolume";
    let Some(suffix) = strip_ascii_prefix_case_insensitive(target, PREFIX) else {
        return false;
    };
    !suffix.is_empty()
        && suffix
            .iter()
            .all(|unit| *unit >= u16::from(b'0') && *unit <= u16::from(b'9'))
}

#[cfg(any(windows, test))]
fn starts_ascii_case_insensitive(value: &[u16], prefix: &str) -> bool {
    strip_ascii_prefix_case_insensitive(value, prefix).is_some()
}

#[cfg(any(windows, test))]
fn strip_ascii_prefix_case_insensitive<'a>(value: &'a [u16], prefix: &str) -> Option<&'a [u16]> {
    let bytes = prefix.as_bytes();
    if value.len() < bytes.len() {
        return None;
    }
    value
        .iter()
        .zip(bytes)
        .take(bytes.len())
        .all(|(actual, expected)| {
            *actual <= u16::from(u8::MAX) && (*actual as u8).eq_ignore_ascii_case(expected)
        })
        .then_some(&value[bytes.len()..])
}

#[cfg(test)]
const BUS_TYPE_SCSI: u32 = 1;
#[cfg(any(windows, test))]
const BUS_TYPE_ATA: u32 = 3;
#[cfg(test)]
const BUS_TYPE_FIBRE: u32 = 6;
#[cfg(test)]
const BUS_TYPE_RAID: u32 = 8;
#[cfg(test)]
const BUS_TYPE_ISCSI: u32 = 9;
#[cfg(test)]
const BUS_TYPE_SAS: u32 = 10;
#[cfg(any(windows, test))]
const BUS_TYPE_SATA: u32 = 11;
#[cfg(test)]
const BUS_TYPE_VIRTUAL: u32 = 14;
#[cfg(test)]
const BUS_TYPE_FILE_BACKED_VIRTUAL: u32 = 15;
#[cfg(test)]
const BUS_TYPE_SPACES: u32 = 16;
#[cfg(any(windows, test))]
const BUS_TYPE_NVME: u32 = 17;
#[cfg(test)]
const BUS_TYPE_NVMEOF: u32 = 20;

#[cfg(windows)]
const IOCTL_VOLUME_GET_VOLUME_DISK_EXTENTS: u32 = 0x0056_0000;
#[cfg(windows)]
const IOCTL_STORAGE_QUERY_PROPERTY: u32 = 0x002d_1400;

#[cfg(windows)]
const QUERY_BUFFER_UNITS: usize = 1024;
#[cfg(windows)]
const DEPENDENCY_BUFFER_BYTES: usize = 64 * 1024;
#[cfg(windows)]
const EXTENT_BUFFER_BYTES: usize = 64 * 1024;
#[cfg(windows)]
const DESCRIPTOR_BUFFER_BYTES: usize = 4096;
#[cfg(windows)]
const MAX_EXTENTS: usize = 1024;

#[cfg(windows)]
#[repr(C, align(8))]
struct AlignedBuffer<const N: usize>([u8; N]);

#[cfg(windows)]
struct OwnedHandle {
    raw: *mut core::ffi::c_void,
    closed: bool,
}

#[cfg(windows)]
impl OwnedHandle {
    fn close(mut self) -> bool {
        let closed = close_raw_handle(self.raw);
        self.closed = true;
        closed
    }
}

#[cfg(windows)]
impl Drop for OwnedHandle {
    fn drop(&mut self) {
        if !self.closed {
            // This is a best-effort unwind/early-failure fallback. Every path
            // that can produce FixedLocal calls `close`, observes its result,
            // and marks the handle closed before Drop.
            let _already_unknown = close_raw_handle(self.raw);
            self.closed = true;
        }
    }
}

#[cfg(windows)]
fn close_raw_handle(raw: *mut core::ffi::c_void) -> bool {
    unsafe {
        // SAFETY: raw came from one successful CreateFileW call. This helper is
        // reached exactly once for each OwnedHandle, either by explicit
        // observed close or by the already-failing Drop fallback.
        CloseHandle(raw) != 0
    }
}

#[cfg(windows)]
fn query_preliminary(root: DriveRoot) -> PreliminaryObservation {
    let drive_type = unsafe {
        // SAFETY: DriveRoot owns a four-unit NUL-terminated drive root for
        // the duration of the read-only GetDriveTypeW call.
        GetDriveTypeW(root.root.as_ptr())
    };
    let mut buffer = [0u16; QUERY_BUFFER_UNITS];
    let returned = unsafe {
        // SAFETY: DriveRoot owns a NUL-terminated drive device name. The
        // writable output and the reported capacity are exactly equal.
        QueryDosDeviceW(
            root.device.as_ptr(),
            buffer.as_mut_ptr(),
            QUERY_BUFFER_UNITS as u32,
        )
    };
    PreliminaryObservation {
        drive_type: drive_type_observation(drive_type),
        mapping: decode_device_mapping(&buffer, returned),
    }
}

#[cfg(windows)]
fn open_device(name: &[u16]) -> Option<OwnedHandle> {
    const FILE_SHARE_READ: u32 = 0x0000_0001;
    const FILE_SHARE_WRITE: u32 = 0x0000_0002;
    const FILE_SHARE_DELETE: u32 = 0x0000_0004;
    const OPEN_EXISTING: u32 = 3;
    const FILE_ATTRIBUTE_NORMAL: u32 = 0x0000_0080;
    let handle = unsafe {
        // SAFETY: name is synthesized by this crate, NUL-terminated, and
        // remains alive for the call. Desired access is zero and the template
        // and security pointers are null.
        CreateFileW(
            name.as_ptr(),
            0,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            core::ptr::null_mut(),
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            core::ptr::null_mut(),
        )
    };
    (!handle.is_null() && handle != (-1isize as *mut core::ffi::c_void)).then_some(OwnedHandle {
        raw: handle,
        closed: false,
    })
}

#[cfg(windows)]
fn device_io_control(
    handle: &OwnedHandle,
    control_code: u32,
    input: &[u8],
    output: &mut [u8],
) -> Option<usize> {
    let mut returned = 0u32;
    let input_pointer = if input.is_empty() {
        core::ptr::null_mut()
    } else {
        input.as_ptr().cast_mut().cast()
    };
    let success = unsafe {
        // SAFETY: handle is live; the input and output slices remain valid for
        // their exact lengths; returned is writable; no OVERLAPPED is used.
        DeviceIoControl(
            handle.raw,
            control_code,
            input_pointer,
            input.len() as u32,
            output.as_mut_ptr().cast(),
            output.len() as u32,
            &mut returned,
            core::ptr::null_mut(),
        )
    };
    (success != 0 && (returned as usize) <= output.len()).then_some(returned as usize)
}

#[cfg(windows)]
fn query_dependencies(handle: &OwnedHandle) -> QueryState<DependencyObservation> {
    const STORAGE_DEPENDENCY_INFO_VERSION_2: u32 = 2;
    const GET_STORAGE_DEPENDENCY_FLAG_HOST_VOLUMES: u32 = 1;
    let mut buffer = Box::new(AlignedBuffer::<DEPENDENCY_BUFFER_BYTES>(
        [0; DEPENDENCY_BUFFER_BYTES],
    ));
    buffer.0[..4].copy_from_slice(&STORAGE_DEPENDENCY_INFO_VERSION_2.to_ne_bytes());
    let mut size_used = 0u32;
    let status = unsafe {
        // SAFETY: handle is a live volume handle. The aligned buffer is
        // writable for the exact bounded size supplied, begins with the type-2
        // version tag, and size_used remains valid for the call.
        GetStorageDependencyInformation(
            handle.raw,
            GET_STORAGE_DEPENDENCY_FLAG_HOST_VOLUMES,
            DEPENDENCY_BUFFER_BYTES as u32,
            buffer.0.as_mut_ptr().cast(),
            &mut size_used,
        )
    };
    decode_dependency_information(&buffer.0, status, size_used)
}

#[cfg(windows)]
fn decode_dependency_information(
    buffer: &[u8],
    status: u32,
    size_used: u32,
) -> QueryState<DependencyObservation> {
    const STORAGE_DEPENDENCY_INFO_VERSION_2: u32 = 2;
    const HEADER_BYTES: usize = 8;
    const ENTRY_BYTES: usize = core::mem::size_of::<StorageDependencyInfoType2Layout>();
    const MAX_DEPENDENCIES: usize = 1024;
    if status != 0 {
        return QueryState::ApiFailure;
    }
    let size_used = size_used as usize;
    if !(HEADER_BYTES..=buffer.len()).contains(&size_used)
        || read_u32(buffer, 0) != Some(STORAGE_DEPENDENCY_INFO_VERSION_2)
    {
        return QueryState::Partial;
    }
    let Some(count) = read_u32(buffer, 4).map(|value| value as usize) else {
        return QueryState::Partial;
    };
    if count > MAX_DEPENDENCIES {
        return QueryState::Partial;
    }
    let Some(required) = count
        .checked_mul(ENTRY_BYTES)
        .and_then(|bytes| HEADER_BYTES.checked_add(bytes))
    else {
        return QueryState::Partial;
    };
    if size_used < required {
        return QueryState::Partial;
    }
    if count == 0 {
        QueryState::Complete(DependencyObservation::None)
    } else {
        QueryState::Complete(DependencyObservation::Present)
    }
}

#[cfg(windows)]
fn query_extents(handle: &OwnedHandle) -> QueryState<Vec<ExtentObservation>> {
    let mut buffer = Box::new(AlignedBuffer::<EXTENT_BUFFER_BYTES>(
        [0; EXTENT_BUFFER_BYTES],
    ));
    let returned = device_io_control(
        handle,
        IOCTL_VOLUME_GET_VOLUME_DISK_EXTENTS,
        &[],
        &mut buffer.0,
    );
    decode_extent_information(&buffer.0, returned)
}

#[cfg(windows)]
fn decode_extent_information(
    buffer: &[u8],
    returned: Option<usize>,
) -> QueryState<Vec<ExtentObservation>> {
    const EXTENTS_OFFSET: usize = core::mem::offset_of!(VolumeDiskExtentsLayout, extent);
    const EXTENT_SIZE: usize = core::mem::size_of::<DiskExtentLayout>();
    const START_OFFSET: usize = core::mem::offset_of!(DiskExtentLayout, starting_offset);
    const LENGTH_OFFSET: usize = core::mem::offset_of!(DiskExtentLayout, extent_length);
    let Some(returned) = returned else {
        return QueryState::ApiFailure;
    };
    if returned > buffer.len() {
        return QueryState::Partial;
    }
    let view = &buffer[..returned];
    let Some(count) = read_u32(view, 0).map(|value| value as usize) else {
        return QueryState::Partial;
    };
    if count == 0 || count > MAX_EXTENTS {
        return QueryState::Partial;
    }
    let Some(required) = count
        .checked_mul(EXTENT_SIZE)
        .and_then(|bytes| EXTENTS_OFFSET.checked_add(bytes))
    else {
        return QueryState::Partial;
    };
    if returned < required {
        return QueryState::Partial;
    }
    let mut extents = Vec::with_capacity(count);
    for index in 0..count {
        let base = EXTENTS_OFFSET + index * EXTENT_SIZE;
        let (Some(disk_number), Some(starting_offset), Some(extent_length)) = (
            read_u32(view, base),
            read_i64(view, base + START_OFFSET),
            read_i64(view, base + LENGTH_OFFSET),
        ) else {
            return QueryState::Partial;
        };
        extents.push(ExtentObservation {
            disk_number,
            starting_offset,
            extent_length,
        });
    }
    QueryState::Complete(extents)
}

#[cfg(windows)]
fn query_backing_disks(extents: &[ExtentObservation]) -> (QueryState<Vec<DiskObservation>>, bool) {
    let mut numbers = extents
        .iter()
        .map(|extent| extent.disk_number)
        .collect::<Vec<_>>();
    numbers.sort_unstable();
    numbers.dedup();
    if numbers.is_empty() || numbers.len() > MAX_EXTENTS {
        return (QueryState::Partial, true);
    }

    let mut disks = Vec::with_capacity(numbers.len());
    for disk_number in numbers {
        let mut name = vec![
            u16::from(b'\\'),
            u16::from(b'\\'),
            u16::from(b'.'),
            u16::from(b'\\'),
        ];
        name.extend("PhysicalDrive".encode_utf16());
        name.extend(disk_number.to_string().encode_utf16());
        name.push(0);
        let Some(handle) = open_device(&name) else {
            return (QueryState::ApiFailure, true);
        };
        let query = query_disk_descriptor(&handle, disk_number);
        let closed = handle.close();
        if !closed {
            return (QueryState::ApiFailure, false);
        }
        let disk = match query {
            QueryState::Complete(disk) => disk,
            QueryState::ApiFailure => return (QueryState::ApiFailure, true),
            QueryState::Partial => return (QueryState::Partial, true),
        };
        disks.push(disk);
    }
    (QueryState::Complete(disks), true)
}

#[cfg(windows)]
fn query_disk_descriptor(handle: &OwnedHandle, disk_number: u32) -> QueryState<DiskObservation> {
    let query = [0u8; 12];
    let mut buffer = Box::new(AlignedBuffer::<DESCRIPTOR_BUFFER_BYTES>(
        [0; DESCRIPTOR_BUFFER_BYTES],
    ));
    let returned = device_io_control(handle, IOCTL_STORAGE_QUERY_PROPERTY, &query, &mut buffer.0);
    decode_disk_descriptor(&buffer.0, returned, disk_number)
}

#[cfg(windows)]
fn decode_disk_descriptor(
    buffer: &[u8],
    returned: Option<usize>,
    disk_number: u32,
) -> QueryState<DiskObservation> {
    const BUS_OFFSET: usize = core::mem::offset_of!(StorageDeviceDescriptorLayout, bus_type);
    const REMOVABLE_OFFSET: usize =
        core::mem::offset_of!(StorageDeviceDescriptorLayout, removable_media);
    const MINIMUM: usize = core::mem::size_of::<StorageDeviceDescriptorLayout>();
    let Some(returned) = returned else {
        return QueryState::ApiFailure;
    };
    if returned > buffer.len() {
        return QueryState::Partial;
    }
    let view = &buffer[..returned];
    let (Some(version), Some(size), Some(bus_type)) = (
        read_u32(view, 0),
        read_u32(view, 4).map(|value| value as usize),
        read_u32(view, BUS_OFFSET),
    ) else {
        return QueryState::Partial;
    };
    if version < MINIMUM as u32 || size < MINIMUM || size > returned {
        return QueryState::Partial;
    }
    let Some(removable) = view.get(REMOVABLE_OFFSET) else {
        return QueryState::Partial;
    };
    QueryState::Complete(DiskObservation {
        disk_number,
        removable: *removable != 0,
        bus_type,
    })
}

#[cfg(windows)]
fn drive_type_observation(raw: u32) -> DriveTypeObservation {
    match raw {
        0 => DriveTypeObservation::Unknown,
        1 => DriveTypeObservation::MissingRoot,
        2 => DriveTypeObservation::Removable,
        3 => DriveTypeObservation::Fixed,
        4 => DriveTypeObservation::Remote,
        5 => DriveTypeObservation::Optical,
        6 => DriveTypeObservation::RamDisk,
        _ => DriveTypeObservation::Other,
    }
}

#[cfg(windows)]
fn decode_device_mapping(buffer: &[u16], returned: u32) -> QueryState<Vec<u16>> {
    let returned = returned as usize;
    if returned == 0 {
        return QueryState::ApiFailure;
    }
    if returned > buffer.len() {
        return QueryState::Partial;
    }
    let initialized = &buffer[..returned];
    let Some(first_nul) = initialized.iter().position(|unit| *unit == 0) else {
        return QueryState::Partial;
    };
    if first_nul == 0 || initialized[first_nul + 1..].iter().any(|unit| *unit != 0) {
        return QueryState::Partial;
    }
    QueryState::Complete(initialized[..first_nul].to_vec())
}

#[cfg(windows)]
fn read_u32(buffer: &[u8], offset: usize) -> Option<u32> {
    buffer
        .get(offset..offset.checked_add(4)?)?
        .try_into()
        .ok()
        .map(u32::from_ne_bytes)
}

#[cfg(windows)]
fn read_i64(buffer: &[u8], offset: usize) -> Option<i64> {
    buffer
        .get(offset..offset.checked_add(8)?)?
        .try_into()
        .ok()
        .map(i64::from_ne_bytes)
}

#[cfg(windows)]
#[repr(C)]
struct DiskExtentLayout {
    disk_number: u32,
    starting_offset: i64,
    extent_length: i64,
}

#[cfg(windows)]
#[repr(C)]
struct VolumeDiskExtentsLayout {
    count: u32,
    extent: DiskExtentLayout,
}

#[cfg(windows)]
#[repr(C)]
struct GuidLayout {
    data1: u32,
    data2: u16,
    data3: u16,
    data4: [u8; 8],
}

#[cfg(windows)]
#[repr(C)]
struct VirtualStorageTypeLayout {
    device_id: u32,
    vendor_id: GuidLayout,
}

#[cfg(windows)]
#[repr(C)]
struct StorageDependencyInfoType2Layout {
    dependency_type_flags: u32,
    provider_specific_flags: u32,
    virtual_storage_type: VirtualStorageTypeLayout,
    ancestor_level: u32,
    dependency_device_name: *mut u16,
    host_volume_name: *mut u16,
    dependent_volume_name: *mut u16,
    dependent_volume_relative_path: *mut u16,
}

#[cfg(windows)]
#[repr(C)]
struct StorageDeviceDescriptorLayout {
    version: u32,
    size: u32,
    device_type: u8,
    device_type_modifier: u8,
    removable_media: u8,
    command_queueing: u8,
    vendor_id_offset: u32,
    product_id_offset: u32,
    product_revision_offset: u32,
    serial_number_offset: u32,
    bus_type: u32,
    raw_properties_length: u32,
}

#[cfg(windows)]
#[link(name = "Kernel32")]
unsafe extern "system" {
    fn GetDriveTypeW(lp_root_path_name: *const u16) -> u32;
    fn QueryDosDeviceW(lp_device_name: *const u16, lp_target_path: *mut u16, ucch_max: u32) -> u32;
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
}

#[cfg(windows)]
#[link(name = "VirtDisk")]
unsafe extern "system" {
    fn GetStorageDependencyInformation(
        object: *mut core::ffi::c_void,
        flags: u32,
        storage_dependency_info_size: u32,
        storage_dependency_info: *mut core::ffi::c_void,
        size_used: *mut u32,
    ) -> u32;
}

#[cfg(test)]
mod tests {
    use super::{
        BUS_TYPE_ATA, BUS_TYPE_FIBRE, BUS_TYPE_FILE_BACKED_VIRTUAL, BUS_TYPE_ISCSI, BUS_TYPE_NVME,
        BUS_TYPE_NVMEOF, BUS_TYPE_RAID, BUS_TYPE_SAS, BUS_TYPE_SATA, BUS_TYPE_SCSI,
        BUS_TYPE_SPACES, BUS_TYPE_VIRTUAL, DependencyObservation, DiskObservation, DriveLocality,
        DriveRoot, DriveTypeObservation, ExtentObservation, InspectionEvidence,
        PreliminaryObservation, QueryState, classify_evidence,
    };

    fn mapping(text: &str) -> QueryState<Vec<u16>> {
        QueryState::Complete(text.encode_utf16().collect())
    }

    #[cfg(windows)]
    fn put_u32(buffer: &mut [u8], offset: usize, value: u32) {
        buffer[offset..offset + 4].copy_from_slice(&value.to_ne_bytes());
    }

    #[cfg(windows)]
    fn put_i64(buffer: &mut [u8], offset: usize, value: i64) {
        buffer[offset..offset + 8].copy_from_slice(&value.to_ne_bytes());
    }

    fn preliminary() -> PreliminaryObservation {
        PreliminaryObservation {
            drive_type: DriveTypeObservation::Fixed,
            mapping: mapping(r"\Device\HarddiskVolume3"),
        }
    }

    fn evidence(bus_type: u32) -> InspectionEvidence {
        InspectionEvidence {
            before: preliminary(),
            dependency: QueryState::Complete(DependencyObservation::None),
            extents: QueryState::Complete(vec![ExtentObservation {
                disk_number: 0,
                starting_offset: 1_048_576,
                extent_length: 4_194_304,
            }]),
            disks: QueryState::Complete(vec![DiskObservation {
                disk_number: 0,
                removable: false,
                bus_type,
            }]),
            closes: QueryState::Complete(()),
            after: preliminary(),
        }
    }

    #[test]
    fn drive_root_accepts_only_ascii_letters() {
        assert!(DriveRoot::from_ascii_letter(b'C').is_some());
        assert!(DriveRoot::from_ascii_letter(b'z').is_some());
        for malformed in [0, b'0', b':', b'\\', 0xff] {
            assert!(DriveRoot::from_ascii_letter(malformed).is_none());
        }
    }

    #[test]
    fn direct_ata_sata_and_nvme_chains_are_fixed_local() {
        for bus in [BUS_TYPE_ATA, BUS_TYPE_SATA, BUS_TYPE_NVME] {
            assert_eq!(classify_evidence(&evidence(bus)), DriveLocality::FixedLocal);
        }
    }

    #[test]
    fn preliminary_drive_and_mapping_observations_are_never_sufficient() {
        let mut evidence = evidence(BUS_TYPE_NVME);
        evidence.dependency = QueryState::ApiFailure;
        assert_eq!(classify_evidence(&evidence), DriveLocality::Unknown);
    }

    #[test]
    fn every_virtual_or_fabric_dependency_fails_closed() {
        for (label, dependency) in [
            ("VHD", DependencyObservation::Vhd),
            ("VHDX", DependencyObservation::Vhdx),
            ("ISO", DependencyObservation::Iso),
            ("UNC-hosted VHD", DependencyObservation::UncHostedVhd),
        ] {
            let mut evidence = evidence(BUS_TYPE_NVME);
            evidence.dependency = QueryState::Complete(dependency);
            assert_eq!(
                classify_evidence(&evidence),
                DriveLocality::Unknown,
                "{label}"
            );
        }
    }

    #[test]
    fn every_non_direct_bus_and_future_value_fails_closed() {
        for (label, bus) in [
            ("SCSI", BUS_TYPE_SCSI),
            ("Fibre", BUS_TYPE_FIBRE),
            ("RAID", BUS_TYPE_RAID),
            ("iSCSI", BUS_TYPE_ISCSI),
            ("SAS", BUS_TYPE_SAS),
            ("Virtual", BUS_TYPE_VIRTUAL),
            ("FileBackedVirtual", BUS_TYPE_FILE_BACKED_VIRTUAL),
            ("Spaces", BUS_TYPE_SPACES),
            ("NVMe-oF", BUS_TYPE_NVMEOF),
            ("future", 0xfeed),
        ] {
            assert_eq!(
                classify_evidence(&evidence(bus)),
                DriveLocality::Unknown,
                "{label}"
            );
        }
    }

    #[test]
    fn removable_backing_disk_fails_closed() {
        let mut evidence = evidence(BUS_TYPE_SATA);
        let QueryState::Complete(disks) = &mut evidence.disks else {
            unreachable!();
        };
        disks[0].removable = true;
        assert_eq!(classify_evidence(&evidence), DriveLocality::Unknown);
    }

    #[test]
    fn multiple_complete_extents_and_disks_are_accepted() {
        let mut evidence = evidence(BUS_TYPE_ATA);
        evidence.extents = QueryState::Complete(vec![
            ExtentObservation {
                disk_number: 0,
                starting_offset: 0,
                extent_length: 4096,
            },
            ExtentObservation {
                disk_number: 1,
                starting_offset: 4096,
                extent_length: 8192,
            },
            ExtentObservation {
                disk_number: 0,
                starting_offset: 12_288,
                extent_length: 4096,
            },
        ]);
        evidence.disks = QueryState::Complete(vec![
            DiskObservation {
                disk_number: 0,
                removable: false,
                bus_type: BUS_TYPE_ATA,
            },
            DiskObservation {
                disk_number: 1,
                removable: false,
                bus_type: BUS_TYPE_NVME,
            },
        ]);
        assert_eq!(classify_evidence(&evidence), DriveLocality::FixedLocal);
    }

    #[test]
    fn incomplete_or_inconsistent_extent_topology_fails_closed() {
        let cases = [
            vec![],
            vec![ExtentObservation {
                disk_number: 0,
                starting_offset: -1,
                extent_length: 4096,
            }],
            vec![ExtentObservation {
                disk_number: 0,
                starting_offset: 0,
                extent_length: 0,
            }],
        ];
        for extents in cases {
            let mut evidence = evidence(BUS_TYPE_NVME);
            evidence.extents = QueryState::Complete(extents);
            assert_eq!(classify_evidence(&evidence), DriveLocality::Unknown);
        }

        let mut missing = evidence(BUS_TYPE_NVME);
        missing.disks = QueryState::Complete(vec![]);
        assert_eq!(classify_evidence(&missing), DriveLocality::Unknown);

        let mut extra = evidence(BUS_TYPE_NVME);
        let QueryState::Complete(disks) = &mut extra.disks else {
            unreachable!();
        };
        disks.push(DiskObservation {
            disk_number: 7,
            removable: false,
            bus_type: BUS_TYPE_NVME,
        });
        assert_eq!(classify_evidence(&extra), DriveLocality::Unknown);
    }

    #[test]
    fn every_query_failure_and_partial_buffer_fails_closed() {
        for partial in [false, true] {
            let mut dependency = evidence(BUS_TYPE_NVME);
            dependency.dependency = if partial {
                QueryState::Partial
            } else {
                QueryState::ApiFailure
            };
            assert_eq!(classify_evidence(&dependency), DriveLocality::Unknown);

            let mut extents = evidence(BUS_TYPE_NVME);
            extents.extents = if partial {
                QueryState::Partial
            } else {
                QueryState::ApiFailure
            };
            assert_eq!(classify_evidence(&extents), DriveLocality::Unknown);

            let mut disks = evidence(BUS_TYPE_NVME);
            disks.disks = if partial {
                QueryState::Partial
            } else {
                QueryState::ApiFailure
            };
            assert_eq!(classify_evidence(&disks), DriveLocality::Unknown);

            let mut before = evidence(BUS_TYPE_NVME);
            before.before.mapping = if partial {
                QueryState::Partial
            } else {
                QueryState::ApiFailure
            };
            assert_eq!(classify_evidence(&before), DriveLocality::Unknown);

            let mut after = evidence(BUS_TYPE_NVME);
            after.after.mapping = if partial {
                QueryState::Partial
            } else {
                QueryState::ApiFailure
            };
            assert_eq!(classify_evidence(&after), DriveLocality::Unknown);
        }
    }

    #[test]
    fn topology_recheck_must_be_identical() {
        let mut mapping_changed = evidence(BUS_TYPE_NVME);
        mapping_changed.after.mapping = mapping(r"\Device\HarddiskVolume4");
        assert_eq!(classify_evidence(&mapping_changed), DriveLocality::Unknown);

        let mut drive_changed = evidence(BUS_TYPE_NVME);
        drive_changed.after.drive_type = DriveTypeObservation::Remote;
        assert_eq!(classify_evidence(&drive_changed), DriveLocality::Unknown);
    }

    #[test]
    fn close_failure_prevents_fixed_local() {
        let mut evidence = evidence(BUS_TYPE_NVME);
        evidence.closes = QueryState::ApiFailure;
        assert_eq!(classify_evidence(&evidence), DriveLocality::Unknown);
    }

    #[test]
    fn preliminary_remote_substituted_removable_and_unknown_results_remain_closed() {
        for (drive_type, target, expected) in [
            (
                DriveTypeObservation::Fixed,
                r"\Device\Mup\server\share",
                DriveLocality::Remote,
            ),
            (
                DriveTypeObservation::Fixed,
                r"\??\C:\workspace",
                DriveLocality::Substituted,
            ),
            (
                DriveTypeObservation::Removable,
                r"\Device\HarddiskVolume3",
                DriveLocality::Removable,
            ),
            (
                DriveTypeObservation::Optical,
                r"\Device\HarddiskVolume3",
                DriveLocality::Removable,
            ),
            (
                DriveTypeObservation::RamDisk,
                r"\Device\HarddiskVolume3",
                DriveLocality::Removable,
            ),
            (
                DriveTypeObservation::Unknown,
                r"\Device\HarddiskVolume3",
                DriveLocality::Unknown,
            ),
            (
                DriveTypeObservation::MissingRoot,
                r"\Device\HarddiskVolume3",
                DriveLocality::Unknown,
            ),
            (
                DriveTypeObservation::Other,
                r"\Device\HarddiskVolume3",
                DriveLocality::Unknown,
            ),
        ] {
            let mut evidence = evidence(BUS_TYPE_NVME);
            evidence.before = PreliminaryObservation {
                drive_type,
                mapping: mapping(target),
            };
            assert_eq!(classify_evidence(&evidence), expected);
        }
    }

    #[cfg(windows)]
    #[test]
    fn windows_abi_layouts_and_buffer_decoders_are_bounded() {
        use super::{
            DiskExtentLayout, QueryState, StorageDependencyInfoType2Layout,
            StorageDeviceDescriptorLayout, VolumeDiskExtentsLayout, decode_device_mapping,
        };
        assert_eq!(core::mem::size_of::<DiskExtentLayout>(), 24);
        assert_eq!(core::mem::offset_of!(VolumeDiskExtentsLayout, extent), 8);
        assert_eq!(core::mem::size_of::<StorageDependencyInfoType2Layout>(), 64);
        assert_eq!(
            core::mem::offset_of!(StorageDeviceDescriptorLayout, bus_type),
            28
        );
        assert_eq!(core::mem::size_of::<StorageDeviceDescriptorLayout>(), 36);

        assert_eq!(decode_device_mapping(&[0; 4], 0), QueryState::ApiFailure);
        assert_eq!(decode_device_mapping(&[0; 4], 5), QueryState::Partial);
        assert_eq!(
            decode_device_mapping(&[u16::from(b'x'); 4], 4),
            QueryState::Partial
        );
        assert_eq!(
            decode_device_mapping(&[u16::from(b'x'), 0, u16::from(b'y'), 0], 4),
            QueryState::Partial
        );
    }

    #[cfg(windows)]
    #[test]
    fn dependency_decoder_rejects_api_and_variable_length_boundary_failures() {
        use super::{
            DependencyObservation, QueryState, StorageDependencyInfoType2Layout,
            decode_dependency_information,
        };
        let entry_bytes = core::mem::size_of::<StorageDependencyInfoType2Layout>();
        let mut none = vec![0u8; 8];
        put_u32(&mut none, 0, 2);
        assert_eq!(
            decode_dependency_information(&none, 0, 8),
            QueryState::Complete(DependencyObservation::None)
        );
        assert_eq!(
            decode_dependency_information(&none, 5, 8),
            QueryState::ApiFailure
        );
        assert_eq!(
            decode_dependency_information(&none[..7], 0, 7),
            QueryState::Partial
        );
        assert_eq!(
            decode_dependency_information(&none, 0, 9),
            QueryState::Partial
        );

        let mut wrong_version = none.clone();
        put_u32(&mut wrong_version, 0, 1);
        assert_eq!(
            decode_dependency_information(&wrong_version, 0, 8),
            QueryState::Partial
        );

        let mut wrong_count = none.clone();
        put_u32(&mut wrong_count, 4, 1);
        assert_eq!(
            decode_dependency_information(&wrong_count, 0, 8),
            QueryState::Partial
        );
        put_u32(&mut wrong_count, 4, 1025);
        assert_eq!(
            decode_dependency_information(&wrong_count, 0, 8),
            QueryState::Partial
        );

        let mut present = vec![0u8; 8 + entry_bytes];
        put_u32(&mut present, 0, 2);
        put_u32(&mut present, 4, 1);
        assert_eq!(
            decode_dependency_information(&present, 0, present.len() as u32),
            QueryState::Complete(DependencyObservation::Present)
        );
    }

    #[cfg(windows)]
    #[test]
    fn extent_decoder_rejects_api_truncation_oversize_and_inconsistent_count() {
        use super::{
            DiskExtentLayout, QueryState, VolumeDiskExtentsLayout, decode_extent_information,
        };
        let base = core::mem::offset_of!(VolumeDiskExtentsLayout, extent);
        let extent_size = core::mem::size_of::<DiskExtentLayout>();
        let start = core::mem::offset_of!(DiskExtentLayout, starting_offset);
        let length = core::mem::offset_of!(DiskExtentLayout, extent_length);
        let mut valid = vec![0u8; base + extent_size];
        put_u32(&mut valid, 0, 1);
        put_u32(&mut valid, base, 7);
        put_i64(&mut valid, base + start, 4096);
        put_i64(&mut valid, base + length, 8192);
        let decoded = decode_extent_information(&valid, Some(valid.len()));
        assert!(matches!(decoded, QueryState::Complete(ref rows) if rows.len() == 1));
        assert_eq!(
            decode_extent_information(&valid, None),
            QueryState::ApiFailure
        );
        assert_eq!(
            decode_extent_information(&valid, Some(3)),
            QueryState::Partial
        );
        assert_eq!(
            decode_extent_information(&valid, Some(valid.len() + 1)),
            QueryState::Partial
        );

        let mut zero = valid.clone();
        put_u32(&mut zero, 0, 0);
        assert_eq!(
            decode_extent_information(&zero, Some(zero.len())),
            QueryState::Partial
        );
        let mut inconsistent = valid.clone();
        put_u32(&mut inconsistent, 0, 2);
        assert_eq!(
            decode_extent_information(&inconsistent, Some(inconsistent.len())),
            QueryState::Partial
        );
    }

    #[cfg(windows)]
    #[test]
    fn descriptor_decoder_rejects_api_wrong_version_and_inconsistent_lengths() {
        use super::{
            BUS_TYPE_NVME, QueryState, StorageDeviceDescriptorLayout, decode_disk_descriptor,
        };
        let minimum = core::mem::size_of::<StorageDeviceDescriptorLayout>();
        let bus = core::mem::offset_of!(StorageDeviceDescriptorLayout, bus_type);
        let removable = core::mem::offset_of!(StorageDeviceDescriptorLayout, removable_media);
        let mut valid = vec![0u8; minimum];
        put_u32(&mut valid, 0, minimum as u32);
        put_u32(&mut valid, 4, minimum as u32);
        put_u32(&mut valid, bus, BUS_TYPE_NVME);
        valid[removable] = 0;
        assert!(matches!(
            decode_disk_descriptor(&valid, Some(valid.len()), 4),
            QueryState::Complete(_)
        ));
        assert_eq!(
            decode_disk_descriptor(&valid, None, 4),
            QueryState::ApiFailure
        );
        assert_eq!(
            decode_disk_descriptor(&valid, Some(minimum - 1), 4),
            QueryState::Partial
        );
        assert_eq!(
            decode_disk_descriptor(&valid, Some(minimum + 1), 4),
            QueryState::Partial
        );

        let mut wrong_version = valid.clone();
        put_u32(&mut wrong_version, 0, (minimum - 1) as u32);
        assert_eq!(
            decode_disk_descriptor(&wrong_version, Some(minimum), 4),
            QueryState::Partial
        );
        let mut wrong_size = valid.clone();
        put_u32(&mut wrong_size, 4, (minimum + 1) as u32);
        assert_eq!(
            decode_disk_descriptor(&wrong_size, Some(minimum), 4),
            QueryState::Partial
        );
        put_u32(&mut wrong_size, 4, (minimum - 1) as u32);
        assert_eq!(
            decode_disk_descriptor(&wrong_size, Some(minimum), 4),
            QueryState::Partial
        );
    }

    #[cfg(not(windows))]
    #[test]
    fn non_windows_classification_is_unsupported_without_foreign_calls() {
        let root = DriveRoot::from_ascii_letter(b'C').expect("drive root");
        assert_eq!(super::classify(root), DriveLocality::Unsupported);
    }
}
