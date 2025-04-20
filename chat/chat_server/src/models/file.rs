use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::AppError;

use super::ChatFile;

use sha1::{Digest, Sha1};
use tracing::info;

impl ChatFile {
    pub fn new(ws_id: i64, filename: &str, data: &[u8]) -> Self {
        let hash = Sha1::digest(data);
        Self {
            ws_id,
            ext: filename.rsplit('.').next().unwrap_or("txt").to_string(),
            hash: hex::encode(hash),
        }
    }

    pub fn url(&self) -> String {
        format!("/files/{}/{}", self.ws_id, self.hash_to_path())
    }
    pub fn path(&self, base_dir: &Path) -> PathBuf {
        base_dir.join(format!("{}/{}", self.ws_id, self.hash_to_path()))
    }

    //split hash into 3 parts, first 2 and 3 chars
    pub fn hash_to_path(&self) -> String {
        let (first, rest) = self.hash.split_at(3);
        let (second, third) = rest.split_at(3);
        info!(self.ext);
        format!("{}/{}/{}.{}", first, second, third, self.ext)
    }
}

impl FromStr for ChatFile {
    type Err = AppError;
    //convert /files/1/0fd/a3e/ed0040e14b47bec49a71f08097b325950d.jpg to ChatFile
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some(s) = s.strip_prefix("/files/") else {
            return Err(AppError::ChatFileError("invalid file path".to_string()));
        };
        let parts: Vec<&str> = s.split('/').collect();
        if parts.len() != 4 {
            return Err(AppError::ChatFileError(
                "File path does not valid".to_string(),
            ));
        }
        let Ok(ws_id) = parts[0].parse::<i64>() else {
            return Err(AppError::ChatFileError("invalid workspace id".to_string()));
        };
        let Some((part3, ext)) = parts[3].split_once('.') else {
            return Err(AppError::ChatFileError("invalid file name".to_string()));
        };

        Ok(Self {
            ws_id,
            ext: ext.to_string(),
            hash: format!("{}{}{}", parts[1], parts[2], part3),
        })
    }
}
