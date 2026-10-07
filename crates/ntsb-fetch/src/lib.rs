//! `ntsb fetch` (spec §4.1): resolve avdata download links from the directory page,
//! download, hash, record in `data/raw/manifest.json`, unzip.

use anyhow::{bail, Context, Result};
use chrono::Utc;
use regex::Regex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

pub const DIRECTORY_URL: &str = "https://data.ntsb.gov/avdata";

/// Files phase 1 needs. Matched case-insensitively against the file name in each link.
pub const WANTED: &[&str] = &[
    "avall.zip",
    "Pre2008.zip",
    "codman.pdf",
    "eadmspub.pdf",
    "eadmspub_legacy.pdf",
    "MDB_Release_Notes.pdf",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestEntry {
    pub file: String,
    pub url: String,
    pub path: String,
    pub bytes: u64,
    pub sha256: String,
    pub last_modified: Option<String>,
    pub etag: Option<String>,
    pub fetched_at: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Manifest {
    pub entries: Vec<ManifestEntry>,
}

impl Manifest {
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let s = fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
        Ok(serde_json::from_str(&s)?)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        fs::write(path, serde_json::to_string_pretty(self)? + "\n")?;
        Ok(())
    }

    fn latest(&self, file: &str) -> Option<&ManifestEntry> {
        self.entries.iter().rev().find(|e| e.file.eq_ignore_ascii_case(file))
    }
}

/// Extract `(file_name, absolute_url)` for every `DownloadFile?fileID=...` link on the page.
pub fn parse_links(html: &str, base: &str) -> Vec<(String, String)> {
    let re = Regex::new(r#"href="(/avdata/FileDirectory/DownloadFile\?fileID=([^"]+))""#).unwrap();
    let origin = base.split("/avdata").next().unwrap_or(base);
    re.captures_iter(html)
        .map(|c| {
            let id = c[2].replace("%3A", ":").replace("%5C", "\\").replace("&amp;", "&");
            let name = id.rsplit('\\').next().unwrap_or(&id).to_string();
            (name, format!("{origin}{}", c[1].replace("&amp;", "&")))
        })
        .collect()
}

pub struct FetchOptions {
    pub raw_dir: PathBuf,
    pub force: bool,
}

pub fn fetch(opts: &FetchOptions) -> Result<Vec<ManifestEntry>> {
    let client = reqwest::blocking::Client::builder()
        .user_agent(concat!("fra-portal-ntsb-fetch/", env!("CARGO_PKG_VERSION")))
        .timeout(Duration::from_secs(1800))
        .build()?;

    let html = client.get(DIRECTORY_URL).send()?.error_for_status()?.text()?;
    let links = parse_links(&html, DIRECTORY_URL);
    if links.is_empty() {
        bail!("no download links found on {DIRECTORY_URL}; page layout may have changed");
    }

    fs::create_dir_all(&opts.raw_dir)?;
    let manifest_path = opts.raw_dir.join("manifest.json");
    let mut manifest = Manifest::load(&manifest_path)?;
    let day_dir = opts.raw_dir.join(Utc::now().format("%Y-%m-%d").to_string());
    let mut fetched = Vec::new();

    for want in WANTED {
        let Some((name, url)) = links.iter().find(|(n, _)| n.eq_ignore_ascii_case(want)) else {
            bail!("{want} not listed on {DIRECTORY_URL}");
        };

        let head = client.head(url).send()?.error_for_status()?;
        let etag = header(&head, "etag");
        let last_modified = header(&head, "last-modified");
        let size = head.content_length();

        if !opts.force {
            if let Some(prev) = manifest.latest(name) {
                let same = (etag.is_some() && prev.etag == etag)
                    || (last_modified.is_some() && prev.last_modified == last_modified)
                    || (etag.is_none() && last_modified.is_none() && size == Some(prev.bytes));
                if same && Path::new(&prev.path).exists() {
                    tracing::info!("{name}: unchanged, skipping");
                    continue;
                }
            }
        }

        fs::create_dir_all(&day_dir)?;
        let path = day_dir.join(name);
        tracing::info!("{name}: downloading {url}");
        let (bytes, sha256) = download(&client, url, &path)?;
        tracing::info!("{name}: {bytes} bytes, sha256 {sha256}");

        if name.to_ascii_lowercase().ends_with(".zip") {
            unzip(&path, &day_dir)?;
        }

        let entry = ManifestEntry {
            file: name.clone(),
            url: url.clone(),
            path: path.to_string_lossy().replace('\\', "/"),
            bytes,
            sha256,
            last_modified,
            etag,
            fetched_at: Utc::now().to_rfc3339(),
        };
        manifest.entries.push(entry.clone());
        manifest.save(&manifest_path)?;
        fetched.push(entry);
    }
    Ok(fetched)
}

fn header(resp: &reqwest::blocking::Response, name: &str) -> Option<String> {
    resp.headers().get(name).and_then(|v| v.to_str().ok()).map(str::to_string)
}

fn download(client: &reqwest::blocking::Client, url: &str, path: &Path) -> Result<(u64, String)> {
    let mut resp = client.get(url).send()?.error_for_status()?;
    let expected = resp.content_length();
    let tmp = path.with_file_name(format!("{}.part", path.file_name().unwrap().to_string_lossy()));
    let mut out = File::create(&tmp)?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 1 << 20];
    let mut total = 0u64;
    loop {
        let n = resp.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
        out.write_all(&buf[..n])?;
        total += n as u64;
    }
    out.flush()?;
    drop(out);
    if let Some(expected) = expected {
        if expected != total {
            bail!("{}: got {total} bytes, expected {expected}", path.display());
        }
    }
    fs::rename(&tmp, path)?;
    Ok((total, format!("{:x}", hasher.finalize())))
}

fn unzip(zip_path: &Path, dest: &Path) -> Result<()> {
    let mut archive = zip::ZipArchive::new(File::open(zip_path)?)?;
    for i in 0..archive.len() {
        let mut f = archive.by_index(i)?;
        let Some(rel) = f.enclosed_name() else {
            bail!("{}: unsafe path in archive", zip_path.display());
        };
        let out_path = dest.join(rel);
        if f.is_dir() {
            fs::create_dir_all(&out_path)?;
            continue;
        }
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent)?;
        }
        tracing::info!("  unzip {}", out_path.display());
        io::copy(&mut f, &mut File::create(&out_path)?)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_download_links() {
        let html = r#"<a href="/avdata/FileDirectory/DownloadFile?fileID=C%3A%5Cavdata%5Cavall.zip">x</a>
            <a href="/avdata/FileDirectory/DownloadFile?fileID=C%3A%5Cavdata%5Ccodman.pdf">y</a>"#;
        let links = parse_links(html, DIRECTORY_URL);
        assert_eq!(links.len(), 2);
        assert_eq!(links[0].0, "avall.zip");
        assert_eq!(
            links[0].1,
            "https://data.ntsb.gov/avdata/FileDirectory/DownloadFile?fileID=C%3A%5Cavdata%5Cavall.zip"
        );
        assert_eq!(links[1].0, "codman.pdf");
    }
}
