use std::{error::Error, fmt::Display, path::Path, thread::sleep, time};

use log::{debug, info};
use quick_xml::se::to_string;
use serde::{Deserialize, Serialize};
use virt::{
    connect::Connect, domain::Domain as VirtDomain, storage_pool::StoragePool,
    storage_vol::StorageVol, sys::VIR_DOMAIN_INTERFACE_ADDRESSES_SRC_LEASE,
};

#[derive(Serialize, Deserialize, Debug)]
struct Empty {}

#[derive(Serialize, Deserialize, Debug)]
pub struct Os {
    #[serde(rename = "type")]
    pub os_type: OsType,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OsType {
    #[serde(rename = "@arch")]
    pub arch: String,
    #[serde(rename = "@machine")]
    pub machine: String,
    #[serde(rename = "$value")]
    pub value: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename = "vcpu")]
pub struct Vcpu {
    #[serde(rename = "$text")]
    count: u64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename = "memory")]
pub struct Memory {
    #[serde(rename = "@unit")]
    unit: String,
    count: u64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename = "driver")]
pub struct Driver {
    #[serde(rename = "@name")]
    name: String,
    #[serde(rename = "@type")]
    driver_type: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename = "source")]
pub struct DiskSource {
    #[serde(rename = "@pool")]
    pool: Option<String>,
    #[serde(rename = "@volume")]
    volume: Option<String>,
    #[serde(rename = "@file")]
    file: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename = "target")]
pub struct DiskTarget {
    #[serde(rename = "@dev")]
    dev: String,
    #[serde(rename = "@bus")]
    bus: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename = "disk")]
pub struct Disk {
    #[serde(rename = "@type")]
    disk_type: String,
    #[serde(rename = "@device")]
    #[serde(skip_serializing_if = "Option::is_none")]
    device: Option<String>,
    driver: Driver,
    source: DiskSource,
    target: DiskTarget,
    #[serde(skip_serializing_if = "Option::is_none")]
    readonly: Option<Empty>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename = "source")]
pub struct InterfaceSource {
    #[serde(rename = "@network")]
    network: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename = "model")]
pub struct InterfaceModel {
    #[serde(rename = "@type")]
    model_type: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename = "disk")]
pub struct Interface {
    #[serde(rename = "@type")]
    interface_type: String,
    source: InterfaceSource,
    model: InterfaceModel,
}

#[derive(Debug, Deserialize, Serialize)]
struct Graphics {
    #[serde(rename = "@type")]
    graphics_type: String,
    #[serde(rename = "@autoport")]
    autoport: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename = "devices")]
pub struct Devices {
    #[serde(default)]
    disk: Vec<Disk>,

    #[serde(default)]
    interface: Vec<Interface>,
    graphics: Vec<Graphics>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename = "domain")]
pub struct Domain {
    #[serde(rename = "@type")]
    domain_type: String,
    name: String,
    os: Os,
    vcpu: Vcpu,
    memory: Memory,
    devices: Devices,
}

#[derive(Debug)]
enum VmError {
    IpAddressTimeout,
}

impl Display for VmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VmError::IpAddressTimeout => write!(f, "IP address wait timeout"),
        }
    }
}

impl Error for VmError {}

pub fn create_vm(
    conn: &Connect,
    name: &str,
    pool: &StoragePool,
    volume: &StorageVol,
    cloud_init: &Path,
) -> Result<VirtDomain, Box<dyn Error>> {
    debug!(
        "Creating new VM {} on storage pool {} and volume {}",
        name,
        pool.get_name()?,
        volume.get_name()?
    );

    let dom = Domain {
        domain_type: "kvm".into(),
        name: name.into(),
        os: Os {
            os_type: OsType {
                arch: "x86_64".into(),
                machine: "q35".into(),
                value: "hvm".into(),
            },
        },
        vcpu: Vcpu { count: 1 },
        memory: Memory {
            unit: "G".into(),
            count: 2,
        },
        devices: Devices {
            disk: vec![
                Disk {
                    disk_type: "volume".into(),
                    device: None,
                    driver: Driver {
                        name: "qemu".into(),
                        driver_type: "qcow2".into(),
                    },
                    source: DiskSource {
                        pool: Some(pool.get_name()?),
                        volume: Some(volume.get_name()?),
                        file: None,
                    },
                    target: DiskTarget {
                        dev: "vda".into(),
                        bus: "virtio".into(),
                    },
                    readonly: None,
                },
                Disk {
                    disk_type: "file".into(),
                    device: Some("cdrom".into()),
                    driver: Driver {
                        name: "qemu".into(),
                        driver_type: "raw".into(),
                    },
                    source: DiskSource {
                        pool: None,
                        volume: None,
                        file: Some(cloud_init.display().to_string()),
                    },
                    target: DiskTarget {
                        dev: "sda".into(),
                        bus: "sata".into(),
                    },
                    readonly: Some(Empty {}),
                },
            ],
            interface: vec![Interface {
                interface_type: "network".into(),
                source: InterfaceSource {
                    // TODO: revisit what network to use
                    network: "default".into(),
                },
                model: InterfaceModel {
                    model_type: "virtio".into(),
                },
            }],
            graphics: vec![Graphics {
                graphics_type: "spice".into(),
                autoport: "yes".into(),
            }],
        },
    };

    let xml = to_string(&dom)?;

    debug!("{}", xml);

    let dom = VirtDomain::define_xml(conn, &xml)?;

    debug!("Created {}", dom.get_name()?);
    dom.create()?;

    let mut timeout = 30; // try to fetch IP address of the VM for 30s
    while timeout > 0 {
        let ifaces = dom.interface_addresses(VIR_DOMAIN_INTERFACE_ADDRESSES_SRC_LEASE, 0)?;
        if ifaces.is_empty() {
            timeout -= 1;
            sleep(time::Duration::from_secs(1));
            continue;
        }

        for iface in ifaces {
            for addr in iface.addrs {
                info!("{}", addr.addr)
            }
        }

        break;
    }

    if timeout <= 0 {
        return Err(VmError::IpAddressTimeout.into());
    }

    Ok(dom)
}
