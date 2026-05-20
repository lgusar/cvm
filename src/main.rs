use std::{
    env,
    fs::{exists, remove_file},
    path::PathBuf,
};

use virt::connect::Connect;

use crate::{cloud_init::create_iso_image, domain::create_vm, pool::get_pool, volume::create_file};

mod cloud_init;
mod domain;
mod pool;
mod volume;

// TODO: remove unwrap everywhere
// TODO: parse args
fn main() {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("Bad usage");
    }

    let mut conn =
        Connect::open(Some("qemu:///system")).expect("Unable to connect to the hypervisor");

    let pool = get_pool(&conn).unwrap();

    // TODO: parse args
    let volume = volume::create_volume(&pool, "test.qcow2").unwrap();

    // TODO: parse args
    let user_data = PathBuf::from("/tmp/test/user-data.yml");
    let meta_data = PathBuf::from("/tmp/test/meta-data.yml");

    if exists("/tmp/cvm/cloud-init.iso").unwrap() {
        remove_file("/tmp/cvm/cloud-init.iso").unwrap();
    }

    let cloud_init = create_iso_image(&user_data, &meta_data).unwrap();

    let path = PathBuf::from(&args[1]).canonicalize().unwrap();
    create_file(&conn, &volume, &path).unwrap();
    create_vm(&conn, "test", &pool, &volume, &cloud_init).unwrap();

    conn.close()
        .expect("Failed to close connection to the hypervisor correctly");
}
