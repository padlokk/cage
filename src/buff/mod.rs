//! Chunker module – bounded-memory file processing with resumable checkpoints.
//!
//! This module adapts the research-grade stream chunker artifacts under `docs/ref/chunker/`
//! to Cage's codebase. It provides chunk planning, resumable processing, and optional
//! progress reporting via RSB's `progress` feature.

use std::collections::HashSet;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;

use serde::{Deserialize, Serialize};

use crate::error::{AgeError, AgeResult};

use rsb::progress::{ProgressManager, ProgressStyle, TerminalConfig, TerminalReporter};

/// Default chunk size if the caller does not supply one (64 MiB)
const DEFAULT_CHUNK_SIZE: u64 = 64 * 1024 * 1024;

/// Chunk specification representing a contiguous byte-range in the source file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkSpec {
    pub id: usize,
    pub start: u64,
    pub end: u64,
    pub size: u64,
}

/// Configuration for chunked processing.
#[derive(Debug, Clone)]
pub struct ChunkerConfig {
    /// Desired chunk size. Defaults to 64 MiB.
    pub chunk_size: u64,
    /// Optional directory to store checkpoints (defaults to alongside source file).
    pub checkpoint_dir: Option<PathBuf>,
    /// Enable progress reporting (requires RSB `progress` feature).
    pub enable_progress: bool,
}

impl Default for ChunkerConfig {
    fn default() -> Self {
        Self {
            chunk_size: DEFAULT_CHUNK_SIZE,
            checkpoint_dir: None,
            enable_progress: true,
        }
    }
}

/// Persistent checkpoint for resumable chunk processing.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChunkCheckpoint {
    source_path: PathBuf,
    file_size: u64,
    modified: Option<SystemTime>,
    chunk_size: u64,
    completed_chunks: Vec<usize>,
    bytes_processed: u64,
}

/// Summary returned after processing.
#[derive(Debug, Clone)]
pub struct ChunkProcessingSummary {
    pub total_bytes: u64,
    pub processed_bytes: u64,
    pub chunks_total: usize,
    pub chunks_completed: usize,
    pub checkpoint_cleared: bool,
}

/// Entry point for chunk planning and processing.
#[derive(Debug)]
pub struct FileChunker {
    source: PathBuf,
    total_size: u64,
    chunks: Vec<ChunkSpec>,
    config: ChunkerConfig,
    checkpoint_path: PathBuf,
}

impl FileChunker {
    /// Construct a chunker for the given file. Verifies the file exists and records metadata.
    pub fn new<P: AsRef<Path>>(source: P, config: ChunkerConfig) -> AgeResult<Self> {
        let source = source.as_ref().to_path_buf();
        if !source.exists() {
            return Err(AgeError::file_error(
                "chunker_source",
                source.clone(),
                std::io::Error::new(std::io::ErrorKind::NotFound, "file not found"),
            ));
        }

        let metadata = fs::metadata(&source)
            .map_err(|e| AgeError::file_error("chunker_metadata", source.clone(), e))?;

        let total_size = metadata.len();
        let chunk_size = config.chunk_size.max(1);
        let chunks = Self::plan_chunks(total_size, chunk_size);
        let checkpoint_path = checkpoint_path(&source, config.checkpoint_dir.as_ref());

        Ok(Self {
            source,
            total_size,
            chunks,
            config,
            checkpoint_path,
        })
    }

    /// Return the immutable chunk plan.
    pub fn chunks(&self) -> &[ChunkSpec] {
        &self.chunks
    }

    /// Process the file chunk-by-chunk, passing each chunk to the provided handler.
    ///
    /// The handler is responsible for performing domain-specific work (e.g., encryption,
    /// hashing, manifest generation). If the handler returns an error, processing stops
    /// and the checkpoint remains on disk for later resumption.
    pub fn process<F>(&self, mut handler: F) -> AgeResult<ChunkProcessingSummary>
    where
        F: FnMut(&ChunkSpec, &[u8]) -> AgeResult<()>,
    {
        let mut checkpoint = self.load_checkpoint()?;
        let completed: HashSet<usize> = checkpoint.completed_chunks.iter().copied().collect();

        let mut file = File::open(&self.source)
            .map_err(|e| AgeError::file_error("chunker_open", self.source.clone(), e))?;

        let progress_manager = if self.config.enable_progress {
            let manager = ProgressManager::new();
            let reporter = TerminalReporter::with_config(TerminalConfig {
                use_colors: true,
                use_unicode: true,
                use_stderr: true,
                ..Default::default()
            });
            manager.add_reporter(Arc::new(reporter));
            Some(manager)
        } else {
            None
        };

        let mut progress_task = progress_manager.as_ref().map(|manager| {
            let task = manager.start_task(
                "chunk-processing",
                ProgressStyle::Bar {
                    total: self.chunks.len() as u64,
                },
            );
            task.update_message("Preparing chunks");
            task
        });

        let mut processed_chunks = 0usize;
        let mut processed_bytes = checkpoint.bytes_processed;

        for chunk in &self.chunks {
            if completed.contains(&chunk.id) {
                if let Some(task) = &progress_task {
                    task.update(
                        chunk.id as u64 + 1,
                        &format!("Skipping chunk {} (already processed)", chunk.id),
                    );
                }
                processed_chunks += 1;
                continue;
            }

            if chunk.size > usize::MAX as u64 {
                return Err(AgeError::ConfigurationError {
                    parameter: "chunk_size".into(),
                    value: chunk.size.to_string(),
                    reason: "Chunk size exceeds usize limits".into(),
                });
            }

            file.seek(SeekFrom::Start(chunk.start))
                .map_err(|e| AgeError::file_error("chunker_seek", self.source.clone(), e))?;

            let mut buffer = vec![0u8; chunk.size as usize];
            let bytes_read = file
                .read(&mut buffer)
                .map_err(|e| AgeError::file_error("chunker_read", self.source.clone(), e))?;
            buffer.truncate(bytes_read);

            handler(chunk, &buffer)?;
            processed_chunks += 1;
            processed_bytes = chunk.end + 1;

            checkpoint.completed_chunks.push(chunk.id);
            checkpoint.bytes_processed = processed_bytes;
            self.save_checkpoint(&checkpoint)?;

            if let Some(task) = &progress_task {
                let pct = (processed_chunks as f64 / self.chunks.len() as f64) * 100.0;
                task.update(
                    chunk.id as u64 + 1,
                    &format!("Chunk {} complete ({:.1}%)", chunk.id, pct),
                );
            }
        }

        // Finished successfully – remove checkpoint
        let _ = fs::remove_file(&self.checkpoint_path);

        if let Some(task) = &mut progress_task {
            task.complete("Chunk processing complete");
        }

        Ok(ChunkProcessingSummary {
            total_bytes: self.total_size,
            processed_bytes,
            chunks_total: self.chunks.len(),
            chunks_completed: processed_chunks,
            checkpoint_cleared: true,
        })
    }

    fn load_checkpoint(&self) -> AgeResult<ChunkCheckpoint> {
        if !self.checkpoint_path.exists() {
            return Ok(ChunkCheckpoint {
                source_path: self.source.clone(),
                file_size: self.total_size,
                modified: metadata_modified(&self.source).ok(),
                chunk_size: self.config.chunk_size,
                completed_chunks: Vec::new(),
                bytes_processed: 0,
            });
        }

        let file = File::open(&self.checkpoint_path).map_err(|e| {
            AgeError::file_error("chunker_checkpoint_open", self.checkpoint_path.clone(), e)
        })?;

        let checkpoint: ChunkCheckpoint =
            serde_json::from_reader(file).map_err(|e| AgeError::ConfigurationError {
                parameter: "chunk_checkpoint".into(),
                value: self.checkpoint_path.display().to_string(),
                reason: format!("Invalid JSON: {e}"),
            })?;

        if checkpoint.file_size != self.total_size {
            return Err(AgeError::ConfigurationError {
                parameter: "chunk_checkpoint".into(),
                value: self.checkpoint_path.display().to_string(),
                reason: "Source file size changed since checkpoint".into(),
            });
        }

        Ok(checkpoint)
    }

    fn save_checkpoint(&self, checkpoint: &ChunkCheckpoint) -> AgeResult<()> {
        if let Some(parent) = self.checkpoint_path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                AgeError::file_error("chunker_checkpoint_dir", parent.to_path_buf(), e)
            })?;
        }

        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&self.checkpoint_path)
            .map_err(|e| {
                AgeError::file_error("chunker_checkpoint_write", self.checkpoint_path.clone(), e)
            })?;

        serde_json::to_writer_pretty(file, checkpoint).map_err(|e| AgeError::ConfigurationError {
            parameter: "chunk_checkpoint".into(),
            value: self.checkpoint_path.display().to_string(),
            reason: format!("Serialization error: {e}"),
        })
    }

    fn plan_chunks(total_size: u64, chunk_size: u64) -> Vec<ChunkSpec> {
        let mut chunks = Vec::new();
        if total_size == 0 {
            return chunks;
        }

        let mut start = 0u64;
        let mut id = 0usize;
        while start < total_size {
            let end = (start + chunk_size - 1).min(total_size - 1);
            let size = end - start + 1;
            chunks.push(ChunkSpec {
                id,
                start,
                end,
                size,
            });
            start = end + 1;
            id += 1;
        }
        chunks
    }
}

fn checkpoint_path(source: &Path, checkpoint_dir: Option<&PathBuf>) -> PathBuf {
    if let Some(dir) = checkpoint_dir {
        let mut path = dir.clone();
        let file_name = source
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "cage.chunk".into());
        path.push(format!("{}.checkpoint", file_name));
        path
    } else {
        source.with_extension("cage.chunk")
    }
}

fn metadata_modified(path: &Path) -> AgeResult<SystemTime> {
    fs::metadata(path)
        .map_err(|e| AgeError::file_error("chunker_metadata", path.to_path_buf(), e))?
        .modified()
        .map_err(|e| AgeError::file_error("chunker_metadata", path.to_path_buf(), e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_chunk_planning() {
        let file = NamedTempFile::new().unwrap();
        let mut f = file.reopen().unwrap();
        f.write_all(&vec![b'a'; 10 * 1024]).unwrap();

        let chunker = FileChunker::new(
            file.path(),
            ChunkerConfig {
                chunk_size: 4096,
                checkpoint_dir: None,
                enable_progress: false,
            },
        )
        .unwrap();

        assert_eq!(chunker.chunks().len(), 3);
        assert_eq!(chunker.chunks()[0].start, 0);
        assert_eq!(chunker.chunks()[2].end, 10 * 1024 - 1);
    }

    #[test]
    fn test_chunk_processing() {
        let file = NamedTempFile::new().unwrap();
        {
            let mut writer = file.reopen().unwrap();
            for i in 0..16384 {
                writer.write_all(&[(i % 256) as u8]).unwrap();
            }
        }

        let chunker = FileChunker::new(file.path(), ChunkerConfig::default()).unwrap();
        let mut collected: Vec<u64> = Vec::new();

        let summary = chunker
            .process(|chunk, data| {
                collected.push(chunk.id as u64 + data.len() as u64);
                Ok(())
            })
            .unwrap();

        assert!(summary.checkpoint_cleared);
        assert_eq!(summary.total_bytes, 16384);
        assert_eq!(summary.chunks_total, chunker.chunks().len());
        assert_eq!(collected.len(), chunker.chunks().len());
    }
}
