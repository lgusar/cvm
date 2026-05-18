use std::fs;
use std::path::PathBuf;

use log::debug;
use quick_xml::se::to_string;
use serde::{Deserialize, Serialize};
use virt::connect::Connect;
use virt::storage_pool::StoragePool;
use virt::sys::VIR_STORAGE_POOL_DEFINE_VALIDATE;

#[derive(Debug, Serialize, Deserialize)]
pub struct Target {
    pub path: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "pool")]
pub struct Pool {
    #[serde(rename = "@type")]
    pub pool_type: String,
    pub name: String,
    pub target: Target,
}

pub fn get_pool(conn: &Connect) -> Result<StoragePool, Box<dyn std::error::Error>> {
    debug!("Fetching storage pool cvm");
    let pool = match StoragePool::lookup_by_name(conn, "cvm") {
        Ok(p) => {
            debug!("Found existing storage pool cvm");
            p
        }

        Err(_) => {
            debug!("Storage pool cvm not found. Creating a new one");
            let path = PathBuf::from("/var/lib/cvm");

            fs::exists(&path)?;

            let pool = Pool {
                pool_type: "dir".into(),
                name: "cvm".into(),
                target: Target {
                    path: path.to_str().unwrap().into(),
                },
            };

            let xml = to_string(&pool)?;
            println!("{}", xml);
            StoragePool::define_xml(conn, &xml, VIR_STORAGE_POOL_DEFINE_VALIDATE)?
        }
    };

    if !pool.get_autostart()? {
        pool.set_autostart(true)?;
    }

    if !pool.is_active()? {
        pool.create(0)?;
    }

    Ok(pool)
}
