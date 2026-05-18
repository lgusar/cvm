use log::debug;
use quick_xml::se::to_string;
use serde::{Deserialize, Serialize};
use virt::{
    connect::Connect, domain::Domain as VirtDomain, storage_pool::StoragePool,
    storage_vol::StorageVol,
};

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
    pool: String,
    #[serde(rename = "@volume")]
    volume: String,
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
    driver: Driver,
    source: DiskSource,
    target: DiskTarget,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename = "source")]
pub struct InterfaceSource {
    #[serde(rename = "@network")]
    network: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename = "disk")]
pub struct Interface {
    #[serde(rename = "@type")]
    interface_type: String,
    source: InterfaceSource,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename = "devices")]
pub struct Devices {
    #[serde(default)]
    disk: Vec<Disk>,

    #[serde(default)]
    interface: Vec<Interface>,
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

pub fn create_vm(
    conn: &Connect,
    name: &str,
    pool: &StoragePool,
    volume: &StorageVol,
) -> Result<VirtDomain, Box<dyn std::error::Error>> {
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
            disk: vec![Disk {
                disk_type: "volume".into(),
                driver: Driver {
                    name: "qemu".into(),
                    driver_type: "raw".into(),
                },
                source: DiskSource {
                    pool: pool.get_name()?,
                    volume: volume.get_name()?,
                },
                target: DiskTarget {
                    dev: "vda".into(),
                    bus: "virtio".into(),
                },
            }],
            interface: vec![],
        },
    };

    let xml = to_string(&dom)?;

    debug!("{}", xml);

    let dom = VirtDomain::define_xml(conn, &xml)?;

    debug!("Successfully created {}", dom.get_name()?);
    dom.create()?;

    Ok(dom)
}
