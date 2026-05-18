use std::{env, path::PathBuf};

use virt::connect::Connect;

use crate::{domain::create_vm, pool::get_pool, volume::create_file};

mod domain;
mod pool;
mod volume;

fn main() {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("Bad usage");
    }

    let mut conn =
        Connect::open(Some("qemu:///system")).expect("Unable to connect to the hypervisor");

    let pool = get_pool(&conn).unwrap();

    // TODO: create new image file volume in the storage disk

    let volume = volume::create_volume(&pool, "test.qcow2").unwrap();
    // TODO: create cloud init iso image with cloud-localds

    let path = PathBuf::from(&args[1]).canonicalize().unwrap();
    create_file(&conn, &volume, &path).unwrap();
    create_vm(&conn, "test", &pool, &volume).unwrap();

    conn.close()
        .expect("Failed to close connection to the hypervisor correctly");
}
