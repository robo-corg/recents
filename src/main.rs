extern crate chrono;

use std::fs::{self, Metadata};
use std::time::{SystemTime, UNIX_EPOCH};
use std::path::{Path, PathBuf};
use std::env;

fn try_readdir<P: AsRef<Path>>(path : P) -> Vec<(PathBuf, Metadata)> {
    let result = fs::read_dir(path);

    if !result.is_ok() {
        return Vec::new();
    }

    let entries = result.unwrap();

    return entries.filter_map(|entry| entry.ok()).filter_map(
        |entry| entry.metadata().ok().map_or(None, |metadata| Some((entry.path(), metadata)))
    ).collect();
}

fn most_recent_mtime(path : &Path, metadata : &Metadata) -> Option<SystemTime> {
    let mut dir_list = try_readdir(path);

    let paths_with_subdir_mtime : Vec<(PathBuf, Metadata, Option<SystemTime>)> =
        dir_list.drain(..)
        .map(|(path, metadata)| {
             let mtime : Option<SystemTime> = metadata.modified().ok();
             (path, metadata, mtime)
        })
        .map(|(entry, metadata, mtime)| {
            let modified = std::cmp::max(
                mtime,
                if metadata.is_dir() {
                    most_recent_mtime(entry.as_path(), &metadata)
                } else {
                    None
                }
            );

            (
                entry,
                metadata,
                modified
            )
        }
    ).collect();

    return paths_with_subdir_mtime.iter().map(|&(_, _, mtime)| mtime).fold(
        metadata.modified().ok(),
        |mtime_a, mtime_b| std::cmp::max(mtime_a, mtime_b)
    );
}

fn main() {
    let root_path = env::args().nth(1).unwrap_or(".".to_string());

    let mut root_listing = try_readdir(root_path);

    let mut paths_with_mtimes : Vec<(PathBuf, SystemTime)> = root_listing.drain(..).filter_map(
        |(path, metadata)| {
            if metadata.is_dir() {
                most_recent_mtime(path.as_path(), &metadata).map(|mtime| (path, mtime))
            }
            else {
                None
            }
        }
    ).collect();

    paths_with_mtimes.sort_by_key(|item| item.1);
    paths_with_mtimes.reverse();

    for (path, mtime) in paths_with_mtimes {
        let dur = mtime.duration_since(UNIX_EPOCH).unwrap();
        let datetime = chrono::NaiveDateTime::from_timestamp(dur.as_secs() as i64, dur.subsec_nanos());

        println!("{} {}", path.display(), datetime);
    }
}
