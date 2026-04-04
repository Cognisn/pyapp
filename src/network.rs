use std::io::{Read, Write};

use anyhow::{bail, Context, Result};

use crate::splash;
use crate::terminal;

pub fn download(url: &String, writer: impl Write, description: &str) -> Result<()> {
    let mut response =
        reqwest::blocking::get(url).with_context(|| format!("download failed: {}", url))?;

    let total = response.content_length().unwrap_or(0);
    let pb = terminal::io_progress_bar(format!("Downloading {}", description), total);

    if splash::is_enabled() && total > 0 {
        let mut buf_writer = pb.wrap_write(writer);
        let mut downloaded: u64 = 0;
        let mut buf = [0u8; 8192];
        loop {
            let n = response.read(&mut buf).with_context(|| "download read failed")?;
            if n == 0 {
                break;
            }
            buf_writer.write_all(&buf[..n])?;
            downloaded += n as u64;
            let fraction = downloaded as f32 / total as f32;
            splash::update(
                &format!("Downloading {}... {}%", description, (fraction * 100.0) as u32),
                0.05 + fraction * 0.25,
            );
        }
    } else {
        response.copy_to(&mut pb.wrap_write(writer))?;
    }

    pb.finish_and_clear();

    if response.status().is_success() {
        Ok(())
    } else {
        bail!("download failed: {}, {}", response.status(), url)
    }
}
