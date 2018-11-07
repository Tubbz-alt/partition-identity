//! Find the ID of a device by its path, or find a device path by its ID.

use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

/// Describes a partition identity.
///
/// A device path may be recovered from this.
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct PartitionID {
    pub variant: PartitionSource,
    pub id: String
}

impl PartitionID {
    /// Construct a new `PartitionID` as the given source.
    pub fn new(variant: PartitionSource, id: String) -> Self {
        Self { variant, id }
    }

    /// Construct a new `PartitionID` as a `ID` source.
    pub fn new_id(id: String) -> Self {
        Self::new(PartitionSource::ID, id)
    }

    /// Construct a new `PartitionID` as a `Label` source.
    pub fn new_label(id: String) -> Self {
        Self::new(PartitionSource::Label, id)
    }

    /// Construct a new `PartitionID` as a `UUID` source.
    pub fn new_uuid(id: String) -> Self {
        Self::new(PartitionSource::UUID, id)
    }

    /// Construct a new `PartitionID` as a `PartLabel` source.
    pub fn new_partlabel(id: String) -> Self {
        Self::new(PartitionSource::PartLabel, id)
    }

    /// Construct a new `PartitionID` as a `PartUUID` source.
    pub fn new_partuuid(id: String) -> Self {
        Self::new(PartitionSource::PartUUID, id)
    }

    /// Construct a new `PartitionID` as a `Path` source.
    pub fn new_path(id: String) -> Self {
        Self::new(PartitionSource::Path, id)
    }

    /// Find the device path of this ID.
    pub fn get_device_path(&self) -> Option<PathBuf> {
        from_uuid(&self.id, Self::dir(self.variant))
    }

    /// Find the given source ID of the device at the given path.
    pub fn get_source<P: AsRef<Path>>(variant: PartitionSource, path: P) -> Option<Self> {
        Some(Self {
            variant,
            id: find_uuid(path.as_ref(), Self::dir(variant))?
        })
    }

    /// Find the UUID of the device at the given path.
    pub fn get_uuid<P: AsRef<Path>>(path: P) -> Option<Self> {
        Self::get_source(PartitionSource::UUID, path)
    }

    /// Find the PARTUUID of the device at the given path.
    pub fn get_partuuid<P: AsRef<Path>>(path: P) -> Option<Self> {
        Self::get_source(PartitionSource::PartUUID, path)
    }

    fn dir(variant: PartitionSource) -> fs::ReadDir {
        let idpath = variant.disk_by_path();
        idpath.read_dir().unwrap_or_else(|why| {
            panic!(format!("unable to find {:?}: {}", idpath, why));
        })
    }
}

impl FromStr for PartitionID {
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        if input.starts_with('/') {
            Ok(PartitionID { variant: PartitionSource::Path, id: input.to_owned() })
        } else if input.starts_with("ID=") {
            Ok(PartitionID { variant: PartitionSource::ID, id: input[3..].to_owned() })
        } else if input.starts_with("LABEL=") {
            Ok(PartitionID { variant: PartitionSource::Label, id: input[6..].to_owned() })
        } else if input.starts_with("PARTLABEL=") {
            Ok(PartitionID { variant: PartitionSource::PartLabel, id: input[10..].to_owned() })
        } else if input.starts_with("PARTUUID=") {
            Ok(PartitionID { variant: PartitionSource::PartUUID, id: input[9..].to_owned() })
        } else if input.starts_with("UUID=") {
            Ok(PartitionID { variant: PartitionSource::UUID, id: input[5..].to_owned() })
        } else {
            Err(format!("'{}' is not a valid PartitionID string", input))
        }
    }
}

/// Describes the type of partition identity.
#[derive(Copy, Clone, Debug, Hash, PartialEq)]
pub enum PartitionSource {
    ID,
    Label,
    PartLabel,
    PartUUID,
    Path,
    UUID
}

impl From<PartitionSource> for &'static str {
    fn from(pid: PartitionSource) -> &'static str {
        match pid {
            PartitionSource::ID => "id",
            PartitionSource::Label => "label",
            PartitionSource::PartLabel => "partlabel",
            PartitionSource::PartUUID => "partuuid",
            PartitionSource::Path => "path",
            PartitionSource::UUID => "uuid"
        }
    }
}

impl PartitionSource {
    fn disk_by_path(self) -> PathBuf {
        PathBuf::from(["/dev/disk/by-", <&'static str>::from(self)].concat())
    }
}

fn find_uuid(path: &Path, uuid_dir: fs::ReadDir) -> Option<String> {
    if let Ok(path) = path.canonicalize() {
        for uuid_entry in uuid_dir.filter_map(|entry| entry.ok()) {
            if let Ok(ref uuid_path) = uuid_entry.path().canonicalize() {
                if uuid_path == &path {
                    if let Some(uuid_entry) = uuid_entry.file_name().to_str() {
                        return Some(uuid_entry.into());
                    }
                }
            }
        }
    }

    None
}

fn from_uuid(uuid: &str, uuid_dir: fs::ReadDir) -> Option<PathBuf> {
    for uuid_entry in uuid_dir.filter_map(|entry| entry.ok()) {
        let uuid_entry = uuid_entry.path();
        if let Some(name) = uuid_entry.file_name() {
            if name == uuid {
                if let Ok(uuid_entry) = uuid_entry.canonicalize() {
                    return Some(uuid_entry);
                }
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partition_id_from_str() {
        assert_eq!(
            "/dev/sda1".parse::<PartitionID>(),
            Ok(PartitionID::new_path("/dev/sda1".into()))
        );

        assert_eq!(
            "ID=abcd".parse::<PartitionID>(),
            Ok(PartitionID::new_id("abcd".into()))
        );

        assert_eq!(
            "LABEL=abcd".parse::<PartitionID>(),
            Ok(PartitionID::new_label("abcd".into()))
        );

        assert_eq!(
            "PARTLABEL=abcd".parse::<PartitionID>(),
            Ok(PartitionID::new_partlabel("abcd".into()))
        );

        assert_eq!(
            "PARTUUID=abcd".parse::<PartitionID>(),
            Ok(PartitionID::new_partuuid("abcd".into()))
        );

        assert_eq!(
            "UUID=abcd".parse::<PartitionID>(),
            Ok(PartitionID::new_uuid("abcd".into()))
        );
    }
}
