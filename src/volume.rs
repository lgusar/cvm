use std::{fs::File, io::Read, path::Path};

use log::debug;
use quick_xml::se::to_string;
use serde::{Deserialize, Serialize};
use virt::{connect::Connect, storage_pool::StoragePool, storage_vol::StorageVol, stream::Stream};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename = "capacity")]
struct Capacity {
    #[serde(rename = "@unit")]
    unit: String,
    #[serde(rename = "$text")]
    capacity: u64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename = "volume")]
struct Volume {
    name: String,
    capacity: Capacity,
}

pub fn create_volume(
    pool: &StoragePool,
    name: &str,
) -> Result<StorageVol, Box<dyn std::error::Error>> {
    debug!("Creating a new storage volume");
    if let Ok(volume) = StorageVol::lookup_by_name(pool, name) {
        debug!("Volume {} already exists, deleting it", name);
        volume.delete(0)?
    }

    let volume = Volume {
        name: name.into(),
        capacity: Capacity {
            unit: "G".into(),
            capacity: 32,
        },
    };

    let xml = to_string(&volume)?;
    let vol = StorageVol::create_xml(pool, &xml, 0)?;

    debug!("Created new volume {}", name);

    Ok(vol)
}

pub fn create_file(
    conn: &Connect,
    volume: &StorageVol,
    path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;

    debug!(
        "copying file {} to volume {}",
        path.display(),
        volume.get_name()?
    );

    let stream = Stream::new(conn, 0)?;
    let length = file.metadata()?.len();
    volume.upload(&stream, 0, length, 0)?;

    let mut remaining = length as usize;
    debug!("to send {} bytes", remaining);
    while remaining > 0 {
        const CHUNK_SIZE: usize = 1024 * 1024;
        let to_read = CHUNK_SIZE.min(remaining);
        let mut buf = vec![0; to_read];

        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }

        let mut offset: usize = 0;
        loop {
            let sent = stream.send(&buf[offset..n])?;
            if sent == 0 {
                return Err("stream closed unexpectedly".into());
            }
            if sent + offset == n {
                break;
            }
            debug!("sent {} bytes to volume", offset);

            offset += sent;
        }

        remaining -= n;
    }

    stream.finish()?;
    Ok(())
}
