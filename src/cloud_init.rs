use std::{
    fs::{File, create_dir, exists},
    io::{Cursor, Read, Write},
    path::{Path, PathBuf},
    sync::Arc,
};

use hadris::iso::{read::PathSeparator::ForwardSlash, write::InputFiles};
use hadris_iso::{
    joliet::JolietLevel,
    write::{
        File as IsoFile, IsoImageWriter,
        options::{CreationFeatures, FormatOptions},
    },
};
use log::debug;

pub fn create_iso_image(
    user_data: &Path,
    meta_data: &Path,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let output_dir = PathBuf::from("/tmp/cvm");
    let mut cloud_init = PathBuf::from(&output_dir);
    cloud_init.push("cloud-init.iso");

    debug!(
        "Generating cloud-init iso image from {} and {} in {}",
        user_data.display(),
        meta_data.display(),
        cloud_init.display()
    );

    let mut user_data_file = File::open(user_data)?;
    debug!("Open user_data {}", user_data.display());
    let mut meta_data_file = File::open(meta_data)?;
    debug!("Open meta_data {}", meta_data.display());

    let mut user_data_content = vec![];
    let mut meta_data_content = vec![];

    user_data_file.read_to_end(&mut user_data_content)?;
    meta_data_file.read_to_end(&mut meta_data_content)?;

    let files = InputFiles {
        path_separator: ForwardSlash,
        files: vec![
            IsoFile::File {
                name: Arc::new("user-data".into()),
                contents: user_data_content,
            },
            IsoFile::File {
                name: Arc::new("meta-data".into()),
                contents: meta_data_content,
            },
        ],
    };

    let features = CreationFeatures {
        joliet: Some(JolietLevel::Level3),
        ..Default::default()
    };

    let options = FormatOptions {
        volume_name: "cidata".to_string(),
        system_id: None,
        volume_set_id: None,
        publisher_id: None,
        preparer_id: None,
        application_id: None,
        features,
        sector_size: 2048,
        path_separator: ForwardSlash,
        strict_charset: false,
    };

    if !exists(&output_dir)? {
        create_dir(&output_dir)?;
    }
    let mut buffer = Cursor::new(vec![0u8; 1024 * 1024]);
    IsoImageWriter::format_new(&mut buffer, files, options)?;

    let mut output = File::create_new(&cloud_init)?;
    output.write_all(buffer.get_ref())?;

    debug!("Created iso image in {}", cloud_init.display());

    Ok(cloud_init)
}
