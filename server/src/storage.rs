use crate::model::{AppError, AppResult, Assessment};
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};
use uuid::Uuid;
#[derive(Clone)]
pub struct Repository {
    root: PathBuf,
}
impl Repository {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }
    pub fn path(&self, id: &str) -> AppResult<PathBuf> {
        let id = Uuid::parse_str(id)
            .map_err(|_| AppError::Validation("Invalid assessment_id UUID".into()))?;
        Ok(self.root.join(format!("{id}.json")))
    }
    pub fn load(&self, id: &str) -> AppResult<Assessment> {
        let p = self.path(id)?;
        let bytes = fs::read(p).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                AppError::NotFound
            } else {
                AppError::Storage
            }
        })?;
        serde_json::from_slice(&bytes).map_err(|_| AppError::Storage)
    }
    pub fn save(&self, a: &Assessment) -> AppResult<()> {
        fs::create_dir_all(&self.root).map_err(|_| AppError::Storage)?;
        let data = serde_json::to_vec_pretty(a).map_err(|_| AppError::Storage)?;
        if data.len() > 128 * 1024 {
            return Err(AppError::Validation(
                "Assessment exceeds the 128 KiB demo limit. Shorten answer text before saving."
                    .into(),
            ));
        }
        let dest = self.path(&a.id)?;
        let temp = self.root.join(format!(".{}.tmp", Uuid::new_v4()));
        let result = (|| {
            let mut f = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temp)?;
            f.write_all(&data)?;
            f.sync_all()?;
            fs::rename(&temp, dest)?;
            Ok::<_, std::io::Error>(())
        })();
        if result.is_err() {
            let _ = fs::remove_file(temp);
            return Err(AppError::Storage);
        }
        Ok(())
    }
    pub fn list(&self) -> AppResult<Vec<Assessment>> {
        if !self.root.exists() {
            return Ok(vec![]);
        }
        let mut out = vec![];
        for e in fs::read_dir(&self.root).map_err(|_| AppError::Storage)? {
            let p = e.map_err(|_| AppError::Storage)?.path();
            if p.extension().is_some_and(|s| s == "json") {
                let b = fs::read(&p).map_err(|_| AppError::Storage)?;
                if let Ok(a) = serde_json::from_slice(&b) {
                    out.push(a)
                }
            }
        }
        out.sort_by(|a: &Assessment, b| a.id.cmp(&b.id));
        Ok(out)
    }
    pub fn delete(&self, id: &str) -> AppResult<()> {
        fs::remove_file(self.path(id)?).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                AppError::NotFound
            } else {
                AppError::Storage
            }
        })
    }
}
pub fn data_dir() -> PathBuf {
    std::env::var_os("ENISA_MCP_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| Path::new("./data").into())
}
