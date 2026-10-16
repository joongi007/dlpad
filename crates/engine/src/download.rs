//! Synchronous worker adapter for explicit external yt-dlp installations.
mod naming;
mod publication;
pub use naming::FinalNameStyle;
mod subtitles;
use crate::process::{self, ExecutionResult, ProcessOutcome, ProcessSpec};
use dlpad_core::{
    jobs::{Job, JobState, VerifiedArtifact},
    options::DownloadOptions,
};
use rusqlite::{Connection, OptionalExtension, params};
use std::{
    ffi::OsString,
    fmt, fs, io,
    path::{Path, PathBuf},
    sync::{Arc, atomic::AtomicBool},
    time::Duration,
};
#[derive(Debug)]
pub enum DownloadError {
    SessionCleanupDeferred,
    Io(io::Error),
    Database(rusqlite::Error),
    Json(serde_json::Error),
    Policy(&'static str),
    ToolDiscovery(&'static str),
    DenoDiscovery(io::Error),
    Execution {
        stage: &'static str,
        result: ExecutionResult,
    },
}
impl fmt::Display for DownloadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SessionCleanupDeferred => {
                f.write_str("process cleanup unconfirmed; session copy must be retained")
            }
            Self::Io(e) => e.fmt(f),
            Self::DenoDiscovery(e) => write!(f, "selected Deno discovery: {e}"),
            Self::Database(e) => e.fmt(f),
            Self::Json(e) => e.fmt(f),
            Self::Policy(e) | Self::ToolDiscovery(e) => f.write_str(e),
            Self::Execution { stage, result } => {
                write!(f, "{stage}: {:?} {:?}", result.outcome, result.exit_code)
            }
        }
    }
}
impl std::error::Error for DownloadError {}
impl From<io::Error> for DownloadError {
    fn from(e: io::Error) -> Self {
        if process::cleanup_is_unconfirmed(&e) {
            Self::SessionCleanupDeferred
        } else {
            Self::Io(e)
        }
    }
}
impl From<rusqlite::Error> for DownloadError {
    fn from(e: rusqlite::Error) -> Self {
        Self::Database(e)
    }
}
impl From<serde_json::Error> for DownloadError {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}
fn deno_discovery_error(error: io::Error) -> DownloadError {
    if process::cleanup_is_unconfirmed(&error) {
        DownloadError::SessionCleanupDeferred
    } else {
        DownloadError::DenoDiscovery(error)
    }
}
/// Startup never creates a new journal or follows a selected journal symlink.
pub fn recover_at_startup(data: &Path) -> Result<usize, DownloadError> {
    let path = data.join("jobs.sqlite");
    match fs::symlink_metadata(&path) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(0),
        Err(error) => return Err(error.into()),
        Ok(metadata) if !metadata.is_file() || metadata.file_type().is_symlink() => {
            return Err(DownloadError::Policy("invalid startup journal path"));
        }
        Ok(_) => {}
    }
    DownloadRepository::open(&path)?.recover()
}
pub struct DownloadRepository {
    connection: Connection,
}
pub const HISTORY_PAGE_SIZE: usize = 128;
#[derive(Debug, Clone)]
pub struct JobSummary {
    pub id: String,
    pub state: JobState,
    pub attempts: usize,
    pub resumable: bool,
    pub tool_version: Option<String>,
}
/// Execution-only capabilities; never serialized into job options or argv snapshots.
pub struct DownloadSession<'a> {
    /// Execution-only original managed-Deno authority; never persisted.
    #[cfg(windows)]
    pub runtime_environment: Option<&'a crate::runtime_environment::WindowsManagedDenoEnvironment>,
    pub cancel: &'a Arc<AtomicBool>,
    pub pause: Option<&'a Arc<AtomicBool>>,
    pub progress: Option<crate::metrics::ProgressFeed>,
    pub cookies: Option<&'a crate::cookies::SessionCookieLease>,
    /// Windows managed reader capability; mixed cookie readers retain their separate gate.
    pub managed: Option<&'a crate::tools::ManagedExecution>,
}
/// Authentication output is omitted, rather than promising arbitrary secret redaction.
pub const SESSION_OUTPUT_OMITTED: &str = "[authenticated tool output omitted]";

pub struct DownloadResult {
    pub job: Job,
    pub tool_version: String,
    pub ffmpeg_version: Option<String>,
    pub deno_version: Option<String>,
    pub metadata: serde_json::Value,
    pub execution: ExecutionResult,
}
fn snapshot(options: &DownloadOptions) -> Result<String, DownloadError> {
    let plan = options
        .plan()
        .map_err(|_| DownloadError::Policy("invalid options"))?;
    let argv: Vec<_> = plan
        .args
        .iter()
        .map(|a| {
            a.to_str()
                .ok_or(DownloadError::Policy("snapshot requires Unicode argv"))
        })
        .collect::<Result<_, _>>()?;
    naming::snapshot_argv(
        &argv,
        naming::style_from_options_json(
            &options
                .to_stored_json()
                .map_err(|_| DownloadError::Policy("invalid stored options"))?,
        )?,
    )
}
fn valid_history_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 80
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_".contains(&b))
}
impl DownloadRepository {
    /// Inspect an existing journal without creating tables or running migrations.
    pub fn open_readonly(path: &Path) -> Result<Self, DownloadError> {
        let connection = Connection::open_with_flags(
            path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY
                | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX
                | rusqlite::OpenFlags::SQLITE_OPEN_NOFOLLOW,
        )?;
        connection.busy_timeout(Duration::from_secs(5))?;
        Ok(Self { connection })
    }

    pub fn open(path: &Path) -> Result<Self, DownloadError> {
        let connection = Connection::open(path)?;
        connection.busy_timeout(Duration::from_secs(5))?;
        connection.execute_batch("CREATE TABLE IF NOT EXISTS download_jobs(id TEXT PRIMARY KEY, revision INTEGER NOT NULL, job_json TEXT NOT NULL, snapshot TEXT NOT NULL, tool_path TEXT, tool_version TEXT)")?;
        // Adapter-local v2 migration avoids changing the shared library PRAGMA user_version.
        let columns: Vec<String> = connection
            .prepare("PRAGMA table_info(download_jobs)")?
            .query_map([], |r| r.get(1))?
            .collect::<Result<_, _>>()?;
        for (name, kind) in [
            ("options_json", "TEXT"),
            ("planned_root", "TEXT"),
            ("planned_path", "TEXT"),
            ("planned_source", "TEXT"),
            ("planned_bytes", "INTEGER"),
            ("planned_artifacts_json", "TEXT"),
        ] {
            if !columns.iter().any(|c| c == name) {
                connection.execute_batch(&format!(
                    "ALTER TABLE download_jobs ADD COLUMN {name} {kind}"
                ))?;
            }
        }
        connection.execute_batch("CREATE TABLE IF NOT EXISTS download_attempts(job_id TEXT NOT NULL,attempt_id TEXT NOT NULL,number INTEGER NOT NULL,tool_path TEXT,tool_version TEXT,PRIMARY KEY(job_id,attempt_id)); CREATE TABLE IF NOT EXISTS download_schema_migrations(version INTEGER PRIMARY KEY); INSERT OR IGNORE INTO download_schema_migrations VALUES(2);")?;
        connection.execute_batch("CREATE TABLE IF NOT EXISTS download_attempt_components(job_id TEXT NOT NULL,attempt_id TEXT NOT NULL,component TEXT NOT NULL,tool_path TEXT NOT NULL,tool_version TEXT NOT NULL,PRIMARY KEY(job_id,attempt_id,component)); INSERT OR IGNORE INTO download_schema_migrations VALUES(3);")?;
        connection.execute_batch("INSERT OR IGNORE INTO download_schema_migrations VALUES(4); INSERT OR IGNORE INTO download_schema_migrations VALUES(5);")?;
        Ok(Self { connection })
    }
    pub fn create_job(&self, id: &str, options: &DownloadOptions) -> Result<Job, DownloadError> {
        if id.is_empty()
            || id.len() > 80
            || !id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        {
            return Err(DownloadError::Policy("invalid job ID"));
        }
        let job = Job::new(id);
        self.connection.execute(
            "INSERT INTO download_jobs(id,revision,job_json,snapshot,options_json) VALUES(?1,0,?2,?3,?4)",
            params![id, serde_json::to_string(&job)?, snapshot(options)?, options.to_stored_json().map_err(|_| DownloadError::Policy("invalid stored options"))?],
        )?;
        Ok(job)
    }
    /// Legacy journals cannot reconstruct typed options safely from argv.
    pub fn stored_options(&self, id: &str) -> Result<DownloadOptions, DownloadError> {
        let (json, saved): (Option<String>, String) = self.connection.query_row(
            "SELECT CASE WHEN length(CAST(options_json AS BLOB))<=131072 THEN options_json ELSE NULL END, snapshot FROM download_jobs WHERE id=?1 AND length(CAST(snapshot AS BLOB))<=131072",
            [id], |row| Ok((row.get(0)?, row.get(1)?)))?;
        let json = json.ok_or(DownloadError::Policy("stored options unavailable"))?;
        let options = DownloadOptions::from_stored_json(&json)
            .map_err(|_| DownloadError::Policy("invalid stored options"))?;
        if snapshot(&options)? != saved {
            return Err(DownloadError::Policy(
                "stored options differ from original snapshot",
            ));
        }
        Ok(options)
    }
    /// Validate a selected history row without materialising an unbounded journal.
    /// Execution revalidates again; this grants no automatic process launch.
    pub fn validated_resume(&self, id: &str) -> Result<(), DownloadError> {
        if !valid_history_id(id) {
            return Err(DownloadError::Policy("invalid history ID"));
        }
        let json: Option<String> = self.connection.query_row(
            "SELECT CASE WHEN length(CAST(job_json AS BLOB))<=131072 THEN job_json ELSE NULL END FROM download_jobs WHERE id=?1",
            [id], |row| row.get(0))?;
        let job: Job =
            serde_json::from_str(&json.ok_or(DownloadError::Policy("history journal too large"))?)?;
        if job.id != id
            || !matches!(
                job.state,
                JobState::Paused | JobState::Failed | JobState::Interrupted
            )
        {
            return Err(DownloadError::Policy("history job is not resumable"));
        }
        self.stored_options(id)?;
        Ok(())
    }
    pub fn history(&self, after: Option<&str>) -> Result<Vec<JobSummary>, DownloadError> {
        self.history_search(after, "")
    }
    /// Literal ID and version-1 StoredOptions URL search, never internal argv or job JSON.
    pub fn history_search(
        &self,
        after: Option<&str>,
        query: &str,
    ) -> Result<Vec<JobSummary>, DownloadError> {
        let query = query.trim();
        if query.len() > 256 || query.contains('\0') {
            return Err(DownloadError::Policy("invalid bounded search query"));
        }
        if after.is_some_and(|id| !valid_history_id(id)) {
            return Err(DownloadError::Policy("invalid history cursor"));
        }
        // CASE prevents oversized/malformed options from reaching JSON extraction.
        // Invalid options are selected as bounded error rows, never converted to empty URL.
        let option_valid = "CASE WHEN options_json IS NULL THEN 1 WHEN typeof(options_json)!='text' OR length(CAST(options_json AS BLOB))>131072 THEN 0 WHEN json_valid(options_json)=0 THEN 0 ELSE COALESCE(json_type(options_json,'$.version')='integer' AND json_extract(options_json,'$.version')=1 AND json_type(options_json,'$.url')='text' AND length(CAST(json_extract(options_json,'$.url') AS BLOB)) BETWEEN 1 AND 8192 AND instr(json_extract(options_json,'$.url'),char(0))=0,0) END";
        let sql = if query.is_empty() {
            "SELECT CASE WHEN length(CAST(id AS BLOB))<=80 THEN id ELSE NULL END,CASE WHEN length(CAST(job_json AS BLOB))<=131072 THEN job_json ELSE NULL END,NULL,1 FROM download_jobs WHERE (?1 IS NULL OR id>?1) ORDER BY id LIMIT 128".to_owned()
        } else {
            format!(
                "SELECT CASE WHEN length(CAST(id AS BLOB))<=80 THEN id ELSE NULL END,CASE WHEN length(CAST(job_json AS BLOB))<=131072 THEN job_json ELSE NULL END,CASE WHEN ({option_valid})=1 THEN options_json ELSE NULL END,({option_valid}) FROM download_jobs WHERE (?1 IS NULL OR id>?1) AND (CASE WHEN ({option_valid})=0 THEN 1 WHEN length(CAST(id AS BLOB))>80 THEN 1 ELSE instr(lower(id),lower(?2))>0 OR instr(lower(CASE WHEN options_json IS NULL THEN '' ELSE json_extract(options_json,'$.url') END),lower(?2))>0 END) ORDER BY id LIMIT 128"
            )
        };
        let mut statement = self.connection.prepare(&sql)?;
        let row_decode = |row: &rusqlite::Row<'_>| {
            Ok((
                row.get::<_, Option<String>>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, bool>(3)?,
            ))
        };
        let rows = if query.is_empty() {
            statement.query_map(params![after], row_decode)?
        } else {
            statement.query_map(params![after, query], row_decode)?
        };
        let mut page = Vec::with_capacity(HISTORY_PAGE_SIZE);
        for row in rows {
            let (id, json, options, valid_options) = row?;
            if !valid_options {
                return Err(DownloadError::Policy(
                    "invalid stored options for history search",
                ));
            }
            if let Some(options) = options {
                DownloadOptions::from_stored_json(&options).map_err(|_| {
                    DownloadError::Policy("invalid stored options for history search")
                })?;
            }
            let json = json.ok_or(DownloadError::Policy("history journal too large"))?;
            let id = id
                .filter(|id| valid_history_id(id))
                .ok_or(DownloadError::Policy("invalid history ID"))?;
            let job: Job = serde_json::from_str(&json)?;
            if job.id != id {
                return Err(DownloadError::Policy("job ID mismatch"));
            }
            let resumable = matches!(
                job.state,
                JobState::Paused | JobState::Failed | JobState::Interrupted
            ) && self.stored_options(&id).is_ok();
            let tool_version = match job.attempts.last() {
                Some(attempt) => self.connection.query_row(
                    "SELECT CASE WHEN length(CAST(tool_version AS BLOB))<=48 THEN tool_version ELSE NULL END FROM download_attempts WHERE job_id=?1 AND attempt_id=?2",
                    params![id, attempt.id], |row| row.get::<_, Option<String>>(0))
                    .optional()?.flatten().filter(|version| !version.is_empty() && version.bytes().all(|b| b.is_ascii_digit() || b"./-+v".contains(&b))),
                None => None,
            };
            page.push(JobSummary {
                id,
                state: job.state,
                attempts: job.attempts.len(),
                resumable,
                tool_version,
            });
        }
        Ok(page)
    }
    pub fn job(&self, id: &str) -> Result<Job, DownloadError> {
        let s: String = self.connection.query_row(
            "SELECT job_json FROM download_jobs WHERE id=?1",
            [id],
            |r| r.get(0),
        )?;
        Ok(serde_json::from_str(&s)?)
    }
    pub fn tool_version(&self, id: &str) -> Result<Option<String>, DownloadError> {
        let job = self.job(id)?;
        match job.attempts.last() {
            Some(attempt) => Ok(self
                .connection
                .query_row(
                    "SELECT tool_version FROM download_attempts WHERE job_id=?1 AND attempt_id=?2",
                    params![id, attempt.id],
                    |r| r.get::<_, Option<String>>(0),
                )
                .optional()?
                .flatten()),
            None => Ok(None),
        }
    }
    pub fn ffmpeg_version(&self, id: &str) -> Result<Option<String>, DownloadError> {
        let job = self.job(id)?;
        match job.attempts.last() {
            Some(attempt) => Ok(self.connection.query_row(
                "SELECT tool_version FROM download_attempt_components WHERE job_id=?1 AND attempt_id=?2 AND component='ffmpeg'",
                params![id, attempt.id], |row| row.get(0)).optional()?),
            None => Ok(None),
        }
    }
    pub fn deno_version(&self, id: &str) -> Result<Option<String>, DownloadError> {
        let job = self.job(id)?;
        match job.attempts.last() {
            Some(attempt) => Ok(self.connection.query_row(
                "SELECT tool_version FROM download_attempt_components WHERE job_id=?1 AND attempt_id=?2 AND component='deno'",
                params![id, attempt.id], |row| row.get(0)).optional()?),
            None => Ok(None),
        }
    }
    fn save(&self, job: &Job, old: u64) -> Result<(), DownloadError> {
        let changed = self.connection.execute(
            "UPDATE download_jobs SET revision=?1,job_json=?2 WHERE id=?3 AND revision=?4",
            params![
                i64::try_from(job.revision)
                    .map_err(|_| DownloadError::Policy("revision overflow"))?,
                serde_json::to_string(job)?,
                job.id,
                i64::try_from(old).map_err(|_| DownloadError::Policy("revision overflow"))?
            ],
        )?;
        if changed != 1 {
            return Err(DownloadError::Policy("job changed concurrently"));
        }
        Ok(())
    }
    fn state(&self, job: &mut Job, state: JobState) -> Result<(), DownloadError> {
        let old = job.revision;
        job.transition(state)
            .map_err(|_| DownloadError::Policy("invalid job transition"))?;
        self.save(job, old)
    }
    pub fn recover(&self) -> Result<usize, DownloadError> {
        let mut stmt = self
            .connection
            .prepare("SELECT job_json FROM download_jobs")?;
        let rows = stmt.query_map([], |r| r.get::<_, String>(0))?;
        let mut count = 0;
        for row in rows {
            let mut job: Job = serde_json::from_str(&row?)?;
            let old = job.revision;
            if job.state == JobState::Verifying {
                let batch: Option<String> = self.connection.query_row(
                    "SELECT CASE WHEN length(CAST(planned_artifacts_json AS BLOB))<=131072 THEN planned_artifacts_json ELSE NULL END FROM download_jobs WHERE id=?1", [&job.id], |row| row.get(0))?;
                let has_batch: bool = self.connection.query_row(
                    "SELECT planned_artifacts_json IS NOT NULL FROM download_jobs WHERE id=?1",
                    [&job.id],
                    |row| row.get(0),
                )?;
                if has_batch {
                    if let Some(json) = batch
                        && let Ok(plan) = publication::Plan::from_json(&json, &job.id)
                        && let Ok(artifacts) = plan.publish(&job.id, true)
                    {
                        job.complete_many(artifacts).map_err(|_| {
                            DownloadError::Policy("invalid batch recovery completion")
                        })?;
                        self.save(&job, old)?;
                        plan.cleanup();
                        count += 1;
                        continue;
                    }
                    if job.recover_interrupted() {
                        self.save(&job, old)?;
                        count += 1;
                    }
                    continue;
                }
                let planned:(Option<String>,Option<String>,Option<String>,Option<i64>)=self.connection.query_row("SELECT planned_root,planned_path,planned_source,planned_bytes FROM download_jobs WHERE id=?1",[&job.id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?)))?;
                if let (Some(root), Some(target), Some(source), Some(bytes)) = planned
                    && let Ok(Some(artifact)) = recover_artifact(
                        &job.id,
                        Path::new(&root),
                        Path::new(&target),
                        Path::new(&source),
                        bytes,
                    )
                {
                    job.complete(artifact)
                        .map_err(|_| DownloadError::Policy("invalid recovery completion"))?;
                    self.save(&job, old)?;
                    count += 1;
                    continue;
                }
            }
            if job.recover_interrupted() {
                self.save(&job, old)?;
                count += 1;
            }
        }
        Ok(count)
    }
    pub fn resume(
        &self,
        id: &str,
        tool: &Path,
        cancel: &Arc<AtomicBool>,
    ) -> Result<DownloadResult, DownloadError> {
        self.resume_with_progress(id, tool, cancel, crate::metrics::ProgressFeed::default())
    }
    pub fn resume_with_progress(
        &self,
        id: &str,
        tool: &Path,
        cancel: &Arc<AtomicBool>,
        progress: crate::metrics::ProgressFeed,
    ) -> Result<DownloadResult, DownloadError> {
        let job = self.job(id)?;
        if !matches!(
            job.state,
            JobState::Paused | JobState::Failed | JobState::Interrupted
        ) {
            return Err(DownloadError::Policy("job is not resumable"));
        }
        let options = self.stored_options(id)?;
        self.execute_with_progress(id, &options, tool, cancel, progress)
    }
    pub fn execute(
        &self,
        id: &str,
        options: &DownloadOptions,
        tool: &Path,
        cancel: &Arc<AtomicBool>,
    ) -> Result<DownloadResult, DownloadError> {
        self.execute_observed(
            id,
            options,
            tool,
            DownloadSession {
                #[cfg(windows)]
                runtime_environment: None,
                cancel,
                pause: None,
                progress: None,
                cookies: None,
                managed: None,
            },
        )
    }
    pub fn execute_with_progress(
        &self,
        id: &str,
        options: &DownloadOptions,
        tool: &Path,
        cancel: &Arc<AtomicBool>,
        progress: crate::metrics::ProgressFeed,
    ) -> Result<DownloadResult, DownloadError> {
        self.execute_observed(
            id,
            options,
            tool,
            DownloadSession {
                #[cfg(windows)]
                runtime_environment: None,
                cancel,
                pause: None,
                progress: Some(progress),
                cookies: None,
                managed: None,
            },
        )
    }
    pub fn execute_with_control(
        &self,
        id: &str,
        options: &DownloadOptions,
        tool: &Path,
        cancel: &Arc<AtomicBool>,
        pause: &Arc<AtomicBool>,
        progress: crate::metrics::ProgressFeed,
    ) -> Result<DownloadResult, DownloadError> {
        self.execute_observed(
            id,
            options,
            tool,
            DownloadSession {
                #[cfg(windows)]
                runtime_environment: None,
                cancel,
                pause: Some(pause),
                progress: Some(progress),
                cookies: None,
                managed: None,
            },
        )
    }
    pub fn resume_with_control(
        &self,
        id: &str,
        tool: &Path,
        cancel: &Arc<AtomicBool>,
        pause: &Arc<AtomicBool>,
        progress: crate::metrics::ProgressFeed,
    ) -> Result<DownloadResult, DownloadError> {
        if !matches!(
            self.job(id)?.state,
            JobState::Paused | JobState::Failed | JobState::Interrupted
        ) {
            return Err(DownloadError::Policy("job is not resumable"));
        }
        let options = self.stored_options(id)?;
        self.execute_with_control(id, &options, tool, cancel, pause, progress)
    }
    pub fn execute_with_session(
        &self,
        id: &str,
        options: &DownloadOptions,
        tool: &Path,
        session: DownloadSession<'_>,
    ) -> Result<DownloadResult, DownloadError> {
        let authenticated = session.cookies.is_some();
        let result = self.execute_observed(id, options, tool, session);
        if authenticated {
            omit_session_result(result)
        } else {
            result
        }
    }
    pub fn resume_with_session(
        &self,
        id: &str,
        tool: &Path,
        session: DownloadSession<'_>,
    ) -> Result<DownloadResult, DownloadError> {
        let authenticated = session.cookies.is_some();
        let result = (|| {
            if !matches!(
                self.job(id)?.state,
                JobState::Paused | JobState::Failed | JobState::Interrupted
            ) {
                return Err(DownloadError::Policy("job is not resumable"));
            }
            let options = self.stored_options(id)?;
            self.execute_observed(id, &options, tool, session)
        })();
        if authenticated {
            omit_session_result(result)
        } else {
            result
        }
    }
    /// Transitional API: Root persists the agreed optional final_name_style in core options.
    pub fn execute_with_session_naming(
        &self,
        id: &str,
        options: &DownloadOptions,
        tool: &Path,
        session: DownloadSession<'_>,
        name_style: Option<FinalNameStyle>,
    ) -> Result<DownloadResult, DownloadError> {
        let authenticated = session.cookies.is_some();
        let result = self.execute_observed_named(id, options, tool, session, name_style);
        if authenticated {
            omit_session_result(result)
        } else {
            result
        }
    }
    pub fn resume_with_session_naming(
        &self,
        id: &str,
        tool: &Path,
        session: DownloadSession<'_>,
        name_style: Option<FinalNameStyle>,
    ) -> Result<DownloadResult, DownloadError> {
        let authenticated = session.cookies.is_some();
        let result = (|| {
            if !matches!(
                self.job(id)?.state,
                JobState::Paused | JobState::Failed | JobState::Interrupted
            ) {
                return Err(DownloadError::Policy("job is not resumable"));
            }
            let options = self.stored_options(id)?;
            self.execute_observed_named(id, &options, tool, session, name_style)
        })();
        if authenticated {
            omit_session_result(result)
        } else {
            result
        }
    }
    fn persisted_name_style(&self, id: &str) -> Result<Option<FinalNameStyle>, DownloadError> {
        let (json,present):(Option<String>,bool)=self.connection.query_row("SELECT CASE WHEN length(CAST(options_json AS BLOB))<=131072 THEN options_json ELSE NULL END,options_json IS NOT NULL FROM download_jobs WHERE id=?1",[id],|r|Ok((r.get(0)?,r.get(1)?)))?;
        match json {
            Some(json) => naming::style_from_options_json(&json),
            None if !present => Ok(None),
            None => Err(DownloadError::Policy("stored filename options limit")),
        }
    }
    fn execute_observed(
        &self,
        id: &str,
        options: &DownloadOptions,
        tool: &Path,
        session: DownloadSession<'_>,
    ) -> Result<DownloadResult, DownloadError> {
        self.execute_observed_named(id, options, tool, session, None)
    }
    fn execute_observed_named(
        &self,
        id: &str,
        options: &DownloadOptions,
        tool: &Path,
        session: DownloadSession<'_>,
        name_style: Option<FinalNameStyle>,
    ) -> Result<DownloadResult, DownloadError> {
        if self.persisted_name_style(id)? != name_style {
            return Err(DownloadError::Policy(
                "job filename options snapshot changed",
            ));
        }
        if name_style.is_some() {
            let has_plan: bool = self.connection.query_row(
                "SELECT planned_artifacts_json IS NOT NULL FROM download_jobs WHERE id=?1",
                [id],
                |r| r.get(0),
            )?;
            if has_plan {
                return Err(DownloadError::Policy(
                    "persisted readable-name publication requires recovery",
                ));
            }
        }

        if naming::style_from_options_json(
            &options
                .to_stored_json()
                .map_err(|_| DownloadError::Policy("invalid stored options"))?,
        )? != name_style
        {
            return Err(DownloadError::Policy(
                "job filename options snapshot changed",
            ));
        }
        // Reject a stale selected capability before recording an attempt or running versions.
        if let Some(lease) = session.cookies {
            lease
                .validate_for_tool()
                .map_err(|_| DownloadError::Policy("private cookie session unavailable"))?;
        }
        let pause = session.pause;
        let mut job = self.job(id)?;
        let saved: String = self.connection.query_row(
            "SELECT snapshot FROM download_jobs WHERE id=?1",
            [id],
            |r| r.get(0),
        )?;
        if snapshot(options)? != saved {
            return Err(DownloadError::Policy("job options snapshot changed"));
        }
        let old = job.revision;
        job.begin_attempt(format!("{id}-{}", job.attempts.len() + 1))
            .map_err(|_| DownloadError::Policy("job already active or terminal"))?;
        self.save(&job, old)?;
        let attempt = job
            .attempts
            .last()
            .ok_or(DownloadError::Policy("missing attempt"))?;
        self.connection.execute(
            "INSERT INTO download_attempts(job_id,attempt_id,number)VALUES(?1,?2,?3)",
            params![job.id, attempt.id, attempt.number],
        )?;
        self.connection.execute("UPDATE download_jobs SET planned_root=NULL,planned_path=NULL,planned_source=NULL,planned_bytes=NULL,planned_artifacts_json=NULL,tool_path=NULL,tool_version=NULL WHERE id=?1",[id])?;
        let result = self.execute_inner(&mut job, options, tool, &session, name_style);
        if let Err(error) = &result {
            // Published-but-uncommitted artifacts remain Verifying for idempotent recovery.
            let planned: Option<String> = self.connection.query_row(
                "SELECT planned_path FROM download_jobs WHERE id=?1",
                [id],
                |r| r.get(0),
            )?;
            if job.state == JobState::Verifying && planned.is_some() {
                return result;
            }
            let cancelled = matches!(error,DownloadError::Execution{result,..}if result.outcome==ProcessOutcome::Cancelled)
                || matches!(error, DownloadError::DenoDiscovery(error) if error.kind()==io::ErrorKind::Interrupted && session.cancel.load(std::sync::atomic::Ordering::Acquire));
            if cancelled {
                self.state(&mut job, JobState::Stopping)?;
                self.state(
                    &mut job,
                    if pause.is_some_and(|intent| intent.load(std::sync::atomic::Ordering::Acquire))
                    {
                        JobState::Paused
                    } else {
                        JobState::Cancelled
                    },
                )?;
            } else {
                self.state(&mut job, JobState::Failed)?;
            }
        }
        result
    }
    fn execute_inner(
        &self,
        job: &mut Job,
        options: &DownloadOptions,
        tool: &Path,
        session: &DownloadSession<'_>,
        name_style: Option<FinalNameStyle>,
    ) -> Result<DownloadResult, DownloadError> {
        let cancel = session.cancel;
        let progress = session.progress.clone();
        #[cfg(windows)]
        if session.runtime_environment.is_some()
            && (options.deno.is_none() || session.managed.is_none())
        {
            return Err(DownloadError::Policy(
                "managed Deno environment requires selected runtime and execution scope",
            ));
        }
        #[cfg(windows)]
        if let Some(policy) = session.runtime_environment {
            session
                .managed
                .ok_or(DownloadError::Policy("managed runtime scope missing"))?
                .validate_runtime_environment(
                    policy,
                    options
                        .deno
                        .as_deref()
                        .ok_or(DownloadError::Policy("managed runtime selection missing"))?,
                    cancel,
                )
                .map_err(|_| DownloadError::Policy("managed runtime original proof changed"))?;
        }
        let executable = fs::canonicalize(tool)?;
        if !executable.is_file() {
            return Err(DownloadError::Policy("tool is not a file"));
        }
        let version = checked(
            &executable,
            vec![
                "--ignore-config".into(),
                "--no-plugin-dirs".into(),
                "--version".into(),
            ],
            Duration::from_secs(10),
            cancel,
            "version",
            None,
            session.managed,
        )?;
        let tool_version = version.logs.trim().to_owned();
        if tool_version.is_empty()
            || tool_version.len() > 48
            || !tool_version
                .bytes()
                .all(|b| b.is_ascii_digit() || b"./-+v".contains(&b))
        {
            return Err(DownloadError::Policy("invalid tool version"));
        }
        self.connection.execute(
            "UPDATE download_jobs SET tool_path=?1,tool_version=?2 WHERE id=?3",
            params![executable.to_string_lossy(), tool_version, job.id],
        )?;
        let attempt = job
            .attempts
            .last()
            .ok_or(DownloadError::Policy("missing attempt"))?;
        self.connection.execute("UPDATE download_attempts SET tool_path=?1,tool_version=?2 WHERE job_id=?3 AND attempt_id=?4",params![executable.to_string_lossy(),tool_version,job.id,attempt.id])?;
        let selected_ffmpeg = options
            .ffmpeg
            .as_ref()
            .map(|path| {
                fs::canonicalize(path).map_err(|_| {
                    DownloadError::ToolDiscovery(
                        "선택한 FFmpeg 파일에 접근할 수 없습니다. 경로와 권한을 확인하세요.",
                    )
                })
            })
            .transpose()?;
        let ffmpeg_version = if let Some(path) = &selected_ffmpeg {
            if !path.is_file() {
                return Err(DownloadError::ToolDiscovery(
                    "FFmpeg 실행 파일을 선택하세요. 폴더는 사용할 수 없습니다.",
                ));
            }
            let version = checked(
                path,
                vec!["-version".into()],
                Duration::from_secs(10),
                cancel,
                "ffmpeg-version",
                None,
                session.managed,
            )?;
            let token = version
                .logs
                .lines()
                .next()
                .and_then(|line| line.strip_prefix("ffmpeg version "))
                .and_then(|line| line.split_ascii_whitespace().next())
                .filter(|value| {
                    !value.is_empty()
                        && value.len() <= 96
                        && value
                            .bytes()
                            .all(|byte| byte.is_ascii_alphanumeric() || b".-_+~".contains(&byte))
                })
                .ok_or(DownloadError::ToolDiscovery(
                    "선택한 파일에서 FFmpeg 버전을 확인하지 못했습니다.",
                ))?;
            self.connection.execute("INSERT INTO download_attempt_components(job_id,attempt_id,component,tool_path,tool_version) VALUES(?1,?2,'ffmpeg',?3,?4)", params![job.id,attempt.id,path.to_string_lossy(),token])?;
            Some(token.to_owned())
        } else {
            None
        };
        let selected_deno = options
            .deno
            .as_ref()
            .map(|path| {
                crate::deno_runtime::inspect_with_managed(
                    path,
                    cancel,
                    session.managed,
                    #[cfg(windows)]
                    session.runtime_environment,
                )
                .map_err(deno_discovery_error)
            })
            .transpose()?;
        let deno_version = if let Some(selected) = &selected_deno {
            self.connection.execute("INSERT INTO download_attempt_components(job_id,attempt_id,component,tool_path,tool_version) VALUES(?1,?2,'deno',?3,?4)",params![job.id,attempt.id,selected.path.to_string_lossy(),selected.version])?;
            Some(selected.version.clone())
        } else {
            None
        };
        let metadata = {
            if let Some(selected) = &selected_deno {
                selected.verify(cancel).map_err(deno_discovery_error)?;
            }
            let metadata_spec = ProcessSpec {
                #[cfg(windows)]
                runtime_environment: session.runtime_environment.cloned(),
                executable: executable.clone(),
                args: {
                    let mut args = vec![
                        "--ignore-config".into(),
                        "--no-plugin-dirs".into(),
                        "--no-playlist".into(),
                        "--dump-single-json".into(),
                    ];
                    if let Some(path) = &selected_ffmpeg {
                        args.extend(["--ffmpeg-location".into(), path.clone().into_os_string()]);
                    }
                    if let Some(selected) = &selected_deno {
                        let mut argument = std::ffi::OsString::from("deno:");
                        argument.push(&selected.path);
                        args.extend(["--no-js-runtimes".into(), "--js-runtimes".into(), argument]);
                    }
                    args.extend(session_cookie_args(session.cookies)?);
                    args.extend(["--".into(), options.url.as_str().into()]);
                    args
                },
                timeout: Duration::from_secs(60),
                max_log_bytes: process::MAX_LOG_BYTES,
            };
            // Protected process gating reserves its own one-shot metadata slot.
            #[cfg(not(any(target_os = "linux", windows)))]
            let reader = begin_cookie_reader(session.cookies)?;
            #[cfg(any(target_os = "linux", windows))]
            let captured = match session.cookies {
                Some(lease) => {
                    #[cfg(windows)]
                    if let Some(managed) = session.managed {
                        managed.run_paired_tool(
                            &metadata_spec,
                            cancel,
                            true,
                            None,
                            crate::tools::ManagedReaderStage::Metadata,
                            lease,
                        )
                    } else {
                        process::run_metadata_with_cookie(&metadata_spec, cancel, lease)
                    }
                    #[cfg(not(windows))]
                    process::run_metadata_with_cookie(&metadata_spec, cancel, lease)
                }
                None => {
                    #[cfg(windows)]
                    if let Some(managed) = session.managed {
                        managed.run_tool(
                            &metadata_spec,
                            cancel,
                            true,
                            None,
                            crate::tools::ManagedReaderStage::Metadata,
                        )
                    } else {
                        process::run_metadata(&metadata_spec, cancel)
                    }
                    #[cfg(not(windows))]
                    process::run_metadata(&metadata_spec, cancel)
                }
            };
            #[cfg(not(any(target_os = "linux", windows)))]
            let captured = process::run_metadata(&metadata_spec, cancel);
            #[cfg(not(any(target_os = "linux", windows)))]
            if !captured
                .as_ref()
                .err()
                .is_some_and(process::cleanup_is_unconfirmed)
                && let Some(reader) = reader
            {
                reader.confirm();
            }
            let captured = captured?;
            if captured.execution.outcome != ProcessOutcome::Exited
                || captured.execution.exit_code != Some(0)
            {
                return Err(DownloadError::Execution {
                    stage: "metadata",
                    result: captured.execution,
                });
            }
            #[derive(serde::Deserialize, serde::Serialize)]
            struct Summary {
                id: Option<String>,
                title: Option<String>,
                extractor: Option<String>,
                duration: Option<f64>,
            }
            // Unknown fields (formats, signed media URLs) are skipped by serde,
            // never retained in the long-running job or ordinary database.
            let summary: Summary = serde_json::from_slice(&captured.stdout)?;
            for (field, max) in [
                (&summary.id, 256),
                (&summary.title, 4096),
                (&summary.extractor, 256),
            ] {
                if field.as_ref().is_some_and(|v| v.len() > max) {
                    return Err(DownloadError::Policy(
                        "metadata summary field limit exceeded",
                    ));
                }
            }
            serde_json::to_value(summary)?
        };
        if let Some(range) = options.time_range
            && let Some(duration) = metadata.get("duration").and_then(serde_json::Value::as_f64)
            && (!duration.is_finite() || duration < 0.0 || range.end_ms as f64 > duration * 1000.0)
        {
            return Err(DownloadError::Policy(
                "time range exceeds known source duration",
            ));
        }

        fs::create_dir_all(&options.output_dir)?;
        let root = fs::canonicalize(&options.output_dir)?;
        let partial_root = root.join(".dlpad-partials");
        ensure_directory(&partial_root)?;
        let partial = partial_root.join(&job.id);
        ensure_directory(&partial)?;
        let partial = fs::canonicalize(partial)?;
        if !partial.starts_with(&root) {
            return Err(DownloadError::Policy("partial path escaped root"));
        }
        if let Some(selected) = &selected_deno {
            selected.verify(cancel).map_err(deno_discovery_error)?;
        }
        let mut resolved_options = options.clone();
        resolved_options.deno = selected_deno.as_ref().map(|selected| selected.path.clone());
        let mut args = resolved_options
            .plan()
            .map_err(|_| DownloadError::Policy("invalid options"))?
            .args;
        if let Some(path) = &selected_ffmpeg {
            let index = args
                .iter()
                .position(|arg| arg == "--ffmpeg-location")
                .ok_or(DownloadError::Policy("missing FFmpeg selection"))?;
            args[index + 1] = path.clone().into_os_string();
        }
        let path_index = args
            .iter()
            .position(|a| a == "--paths")
            .ok_or(DownloadError::Policy("missing managed output path"))?;
        args[path_index + 1] = partial.clone().into_os_string();
        let last = args.len() - 2;
        args.splice(
            last..last,
            [
                "--no-overwrites".into(),
                "--no-simulate".into(),
                "--progress".into(),
                "--progress-delta".into(),
                "1".into(),
                "--progress-template".into(),
                r#"download:DLPAD_PROGRESS:{"speed":%(progress.speed|null)j,"eta":%(progress.eta|null)j,"downloaded":%(progress.downloaded_bytes|null)j,"total":%(progress.total_bytes|null)j,"estimated_total":%(progress.total_bytes_estimate|null)j,"status":%(progress.status|null)j}"#
                    .into(),
                "--output".into(),
                format!("{}.%(ext)s", job.id).into(),
                "--print".into(),
                "after_move:DLPAD_ARTIFACT:%(filepath)j".into(),
            ],
        );
        if options.subtitles.is_some() {
            let last = args.len() - 2;
            args.splice(last..last, ["--print".into(),
                r#"after_video:DLPAD_SUBTITLES:{"id":%(id)j,"requested_subtitles":{"ko":%(requested_subtitles.ko.{filepath,ext}|null)j,"en":%(requested_subtitles.en.{filepath,ext}|null)j}}"#.into()]);
        }
        if options.time_range.is_some() {
            let last = args.len() - 2;
            args.splice(
                last..last,
                [
                    "--external-downloader-args".into(),
                    "ffmpeg:-progress pipe:2 -nostats".into(),
                ],
            );
        }
        if let Some(feed) = &progress {
            let duration_us = processing_duration_us(options.time_range, &metadata);
            feed.set_processing_duration(duration_us);
        }
        #[cfg(target_os = "linux")]
        let _processing_endpoint = progress.as_ref().and_then(|feed| {
            match crate::processing_endpoint::ProcessingEndpoint::new(&partial, feed.clone()) {
                Ok(endpoint) => {
                    let last = args.len() - 2;
                    args.splice(
                        last..last,
                        [
                            "--postprocessor-args".into(),
                            endpoint.postprocessor_argument(),
                        ],
                    );
                    if options.time_range.is_some() {
                        let index = args
                            .iter()
                            .position(|arg| arg == "--external-downloader-args")?;
                        args[index + 1] = endpoint.postprocessor_argument();
                    }
                    Some(endpoint)
                }
                Err(_) => {
                    feed.invalidate_processing();
                    // Optional observation must not prevent an otherwise valid download.
                    None
                }
            }
        });
        let cookie_args = session_cookie_args(session.cookies)?;
        let cookie_index = args.len() - 2;
        args.splice(cookie_index..cookie_index, cookie_args);
        self.state(job, JobState::Downloading)?;
        #[cfg(not(any(target_os = "linux", windows)))]
        let reader = begin_cookie_reader(session.cookies)?;
        let execution = checked_with_cookie(
            ProcessSpec {
                #[cfg(windows)]
                runtime_environment: session.runtime_environment.cloned(),
                executable: executable.clone(),
                args,
                timeout: Duration::from_secs(24 * 60 * 60),
                max_log_bytes: process::MAX_LOG_BYTES,
            },
            cancel,
            "download",
            progress,
            session.cookies,
            session.managed,
        );
        #[cfg(not(any(target_os = "linux", windows)))]
        if !matches!(&execution, Err(DownloadError::SessionCleanupDeferred))
            && let Some(reader) = reader
        {
            reader.confirm();
        }
        #[cfg(target_os = "linux")]
        if let Some(mut endpoint) = _processing_endpoint {
            let _ = endpoint.finish();
        }
        let execution = execution?;
        self.state(job, JobState::Verifying)?;
        let paths: Vec<String> = execution
            .logs
            .lines()
            .filter_map(|l| l.strip_prefix("DLPAD_ARTIFACT:"))
            .map(serde_json::from_str)
            .collect::<Result<_, _>>()?;
        if paths.len() != 1 {
            return Err(DownloadError::Policy("expected exactly one final artifact"));
        }
        let reported = PathBuf::from(&paths[0]);
        if fs::symlink_metadata(&reported)?.file_type().is_symlink() {
            return Err(DownloadError::Policy("artifact symlink rejected"));
        }
        let source = fs::canonicalize(reported)?;
        if source.parent() != Some(partial.as_path()) {
            return Err(DownloadError::Policy(
                "artifact escaped owned partial directory",
            ));
        }
        let stat = fs::metadata(&source)?;
        if !stat.is_file() || stat.len() == 0 {
            return Err(DownloadError::Policy("artifact is empty or not a file"));
        }
        let name = source
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or(DownloadError::Policy("invalid artifact filename"))?;
        if !name.starts_with(&format!("{}.", job.id)) || name.ends_with(".part") {
            return Err(DownloadError::Policy("artifact identity mismatch"));
        }
        if name_style.is_none() {
            let target = root.join(name);
            if target.try_exists()? {
                return Err(DownloadError::Policy("artifact destination already exists"));
            }
            self.connection.execute("UPDATE download_jobs SET planned_root=?1,planned_path=?2,planned_source=?3,planned_bytes=?4 WHERE id=?5 AND revision=?6",params![root.to_str().ok_or(DownloadError::Policy("non-Unicode root"))?,target.to_str().ok_or(DownloadError::Policy("non-Unicode target"))?,source.to_str().ok_or(DownloadError::Policy("non-Unicode source"))?,i64::try_from(stat.len()).map_err(|_|DownloadError::Policy("artifact size overflow"))?,job.id,i64::try_from(job.revision).map_err(|_|DownloadError::Policy("revision overflow"))?])?;
        }
        let first_source = source.clone();
        let mut sources = vec![source];
        if let Some(selected) = options.subtitles {
            let mut markers = execution
                .logs
                .lines()
                .filter_map(|line| line.strip_prefix("DLPAD_SUBTITLES:"));
            let evidence = markers
                .next()
                .ok_or(DownloadError::Policy("missing subtitle evidence"))?;
            if markers.next().is_some() {
                return Err(DownloadError::Policy("duplicate subtitle evidence"));
            }
            let source_id = metadata
                .get("id")
                .and_then(serde_json::Value::as_str)
                .ok_or(DownloadError::Policy("missing subtitle source identity"))?;
            let candidates = subtitles::collect_staging_subtitles(
                &partial,
                &job.id,
                source_id,
                selected,
                evidence.as_bytes(),
            )
            .map_err(|_| DownloadError::Policy("invalid subtitle staging evidence"))?;
            sources.extend(candidates.into_iter().map(|candidate| candidate.path));
        }
        let plan = match name_style {
            None => publication::Plan::prepare(&root, &job.id, &sources)?,
            Some(FinalNameStyle::TitleWithJobId) => publication::Plan::prepare_readable(
                &root,
                &job.id,
                &sources,
                metadata
                    .get("title")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or(""),
            )?,
        };
        if name_style.is_some() {
            self.connection.execute("UPDATE download_jobs SET planned_artifacts_json=?1,planned_root=?2,planned_path=?3,planned_source=?4,planned_bytes=?5 WHERE id=?6 AND revision=?7",params![plan.json()?,root.to_str().ok_or(DownloadError::Policy("non-Unicode root"))?,plan.first_target().to_str().ok_or(DownloadError::Policy("non-Unicode target"))?,first_source.to_str().ok_or(DownloadError::Policy("non-Unicode source"))?,i64::try_from(stat.len()).map_err(|_|DownloadError::Policy("artifact size overflow"))?,job.id,i64::try_from(job.revision).map_err(|_|DownloadError::Policy("revision overflow"))?])?;
        } else {
            self.connection.execute(
                "UPDATE download_jobs SET planned_artifacts_json=?1 WHERE id=?2 AND revision=?3",
                params![
                    plan.json()?,
                    job.id,
                    i64::try_from(job.revision)
                        .map_err(|_| DownloadError::Policy("revision overflow"))?
                ],
            )?;
        }
        let artifacts = plan.publish(&job.id, false)?;
        let old = job.revision;
        let mut completed = job.clone();
        completed
            .complete_many(artifacts)
            .map_err(|_| DownloadError::Policy("invalid artifact evidence"))?;
        self.save(&completed, old)?;
        *job = completed;
        plan.cleanup();
        Ok(DownloadResult {
            job: job.clone(),
            tool_version,
            ffmpeg_version,
            deno_version,
            metadata,
            execution,
        })
    }
}
#[cfg(not(any(target_os = "linux", windows)))]
fn begin_cookie_reader(
    lease: Option<&crate::cookies::SessionCookieLease>,
) -> Result<Option<crate::cookies::SessionCookieReader<'_>>, DownloadError> {
    lease
        .map(crate::cookies::SessionCookieLease::begin_reader)
        .transpose()
        .map_err(|_| DownloadError::Policy("private cookie reader unavailable"))
}

fn session_cookie_args(
    lease: Option<&crate::cookies::SessionCookieLease>,
) -> Result<Vec<OsString>, DownloadError> {
    match lease {
        None => Ok(Vec::new()),
        Some(lease) => {
            lease
                .validate_for_tool()
                .map_err(|_| DownloadError::Policy("private cookie session unavailable"))?;
            Ok(vec![
                "--cookies".into(),
                lease.path_for_tool().as_os_str().to_owned(),
            ])
        }
    }
}
fn omit_session_result(
    result: Result<DownloadResult, DownloadError>,
) -> Result<DownloadResult, DownloadError> {
    match result {
        Ok(mut value) => {
            value.execution.logs = SESSION_OUTPUT_OMITTED.to_owned();
            // Internal ID remains available through subtitle verification/publication.
            value.metadata = serde_json::json!({"duration": value.metadata.get("duration").cloned().unwrap_or(serde_json::Value::Null)});
            Ok(value)
        }
        Err(DownloadError::Execution { stage, mut result }) => {
            result.logs = SESSION_OUTPUT_OMITTED.to_owned();
            Err(DownloadError::Execution { stage, result })
        }
        Err(DownloadError::SessionCleanupDeferred) => Err(DownloadError::SessionCleanupDeferred),
        Err(DownloadError::Io(error)) => Err(DownloadError::Io(io::Error::new(
            error.kind(),
            "authenticated operation failed; details omitted",
        ))),
        Err(DownloadError::DenoDiscovery(error)) => {
            Err(DownloadError::DenoDiscovery(io::Error::new(
                error.kind(),
                "authenticated runtime discovery failed; details omitted",
            )))
        }
        Err(DownloadError::Json(_)) => Err(DownloadError::Policy(
            "authenticated tool evidence invalid; details omitted",
        )),
        Err(DownloadError::Database(_)) => Err(DownloadError::Policy(
            "authenticated journal operation failed; details omitted",
        )),
        Err(error) => Err(error), // Policy/discovery messages are fixed, never tool text.
    }
}

fn recover_artifact(
    id: &str,
    root: &Path,
    target: &Path,
    source: &Path,
    bytes: i64,
) -> io::Result<Option<VerifiedArtifact>> {
    if bytes <= 0 {
        return Ok(None);
    }
    let root = fs::canonicalize(root)?;
    if target.parent() != Some(root.as_path())
        || !target
            .file_name()
            .and_then(|s| s.to_str())
            .is_some_and(|name| name.starts_with(&format!("{id}.")) && !name.ends_with(".part"))
    {
        return Ok(None);
    }
    let target_meta = fs::symlink_metadata(target)?;
    let source_meta = fs::symlink_metadata(source)?;
    if !target_meta.is_file()
        || target_meta.file_type().is_symlink()
        || !source_meta.is_file()
        || source_meta.file_type().is_symlink()
        || target_meta.len() != bytes as u64
        || source_meta.len() != bytes as u64
    {
        return Ok(None);
    }
    let expected_source = root.join(".dlpad-partials").join(id);
    if fs::canonicalize(&expected_source)? != expected_source {
        return Ok(None);
    }
    if fs::canonicalize(source)?.parent() != Some(fs::canonicalize(expected_source)?.as_path()) {
        return Ok(None);
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if target_meta.dev() != source_meta.dev() || target_meta.ino() != source_meta.ino() {
            return Ok(None);
        }
        Ok(Some(VerifiedArtifact {
            path: target.to_owned(),
            bytes: bytes as u64,
        }))
    }
    #[cfg(windows)]
    {
        let source_file = crate::file_identity::open_regular_no_reparse(source)?;
        let target_file = crate::file_identity::open_regular_no_reparse(target)?;
        if crate::file_identity::final_path(&source_file)?.as_path() != source
            || crate::file_identity::final_path(&target_file)?.as_path() != target
            || crate::file_identity::from_file(&source_file)?
                != crate::file_identity::from_file(&target_file)?
        {
            return Ok(None);
        }
        Ok(Some(VerifiedArtifact {
            path: target.to_owned(),
            bytes: bytes as u64,
        }))
    }
    #[cfg(not(any(unix, windows)))]
    {
        Ok(None)
    }
}
fn ensure_directory(path: &Path) -> Result<(), DownloadError> {
    match fs::symlink_metadata(path) {
        Ok(m) if m.is_dir() && !m.file_type().is_symlink() => Ok(()),
        Ok(_) => Err(DownloadError::Policy(
            "partial directory is not an owned directory",
        )),
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            fs::create_dir(path)?;
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}
fn checked(
    tool: &Path,
    args: Vec<OsString>,
    timeout: Duration,
    cancel: &Arc<AtomicBool>,
    stage: &'static str,
    progress: Option<crate::metrics::ProgressFeed>,
    managed: Option<&crate::tools::ManagedExecution>,
) -> Result<ExecutionResult, DownloadError> {
    checked_with_cookie(
        ProcessSpec {
            #[cfg(windows)]
            runtime_environment: None,
            executable: tool.into(),
            args,
            timeout,
            max_log_bytes: process::MAX_LOG_BYTES,
        },
        cancel,
        stage,
        progress,
        None,
        managed,
    )
}

fn checked_with_cookie(
    spec: ProcessSpec,
    cancel: &Arc<AtomicBool>,
    stage: &'static str,
    progress: Option<crate::metrics::ProgressFeed>,
    cookies: Option<&crate::cookies::SessionCookieLease>,
    managed: Option<&crate::tools::ManagedExecution>,
) -> Result<ExecutionResult, DownloadError> {
    #[cfg(any(target_os = "linux", windows))]
    let result = match cookies {
        Some(lease) => {
            #[cfg(windows)]
            if let Some(managed) = managed {
                managed
                    .run_paired_tool(
                        &spec,
                        cancel,
                        false,
                        progress,
                        crate::tools::ManagedReaderStage::Download,
                        lease,
                    )?
                    .execution
            } else {
                process::run_with_cookie(&spec, cancel, progress, lease)?
            }
            #[cfg(not(windows))]
            process::run_with_cookie(&spec, cancel, progress, lease)?
        }
        None => {
            #[cfg(windows)]
            if let Some(managed) = managed {
                let reader_stage = if stage == "download" {
                    crate::tools::ManagedReaderStage::Download
                } else {
                    crate::tools::ManagedReaderStage::VersionProbe
                };
                managed
                    .run_tool(&spec, cancel, false, progress, reader_stage)?
                    .execution
            } else {
                match progress {
                    Some(feed) => process::run_with_progress(&spec, cancel, feed)?,
                    None => process::run(&spec, cancel)?,
                }
            }
            #[cfg(not(windows))]
            match progress {
                Some(feed) => process::run_with_progress(&spec, cancel, feed)?,
                None => process::run(&spec, cancel)?,
            }
        }
    };
    #[cfg(not(any(target_os = "linux", windows)))]
    let result = match progress {
        Some(feed) => process::run_with_progress(&spec, cancel, feed)?,
        None => process::run(&spec, cancel)?,
    };
    #[cfg(not(any(target_os = "linux", windows)))]
    let _ = cookies;
    #[cfg(not(windows))]
    let _ = managed;
    if result.outcome != ProcessOutcome::Exited
        || result.exit_code != Some(0)
        || (result.truncated && stage != "download")
    {
        return Err(DownloadError::Execution { stage, result });
    }
    Ok(result)
}

fn processing_duration_us(
    range: Option<dlpad_core::options::TimeRange>,
    metadata: &serde_json::Value,
) -> Option<u64> {
    match range {
        Some(range) => range
            .end_ms
            .checked_sub(range.start_ms)
            .and_then(|duration| duration.checked_mul(1000)),
        None => metadata
            .get("duration")
            .and_then(serde_json::Value::as_f64)
            .filter(|duration| duration.is_finite() && *duration > 0.0 && *duration < 86400.0)
            .map(|duration| (duration * 1_000_000.0) as u64),
    }
}

#[cfg(test)]
mod portable_history_tests {
    use super::*;
    use dlpad_core::options::{Quality, SourceUrl};

    // History policy is a DB-only contract: no executable or OS signal fixture.
    fn history_fixture() -> (tempfile::TempDir, DownloadOptions, DownloadRepository) {
        let dir = tempfile::tempdir().unwrap();
        let options = DownloadOptions {
            url: SourceUrl::parse("https://example.org/fixture").unwrap(),
            output_dir: dir.path().join("media 한글"),
            quality: Quality::Best,
            time_range: None,
            advanced: vec![],
            ffmpeg: None,
            deno: None,
            final_name_style: None,
            subtitles: None,
        };
        let repo = DownloadRepository::open(&dir.path().join("jobs.sqlite")).unwrap();
        repo.create_job("fixture-job", &options).unwrap();
        (dir, options, repo)
    }
    #[test]
    fn history_search_full_database_url_literals_not_argv_keyset_and_legacy_clear() {
        let (dir, mut options, repo) = history_fixture();
        options.url = SourceUrl::parse("https://example.org/한글_%'clip").unwrap();
        for index in 0..140 {
            repo.create_job(&format!("search-{index:03}"), &options)
                .unwrap();
        }
        repo.create_job("legacy-id", &options).unwrap();
        repo.connection.execute("UPDATE download_jobs SET options_json=NULL,snapshot='LEGACY_ARGV_SECRET' WHERE id='legacy-id'",[]).unwrap();
        let db = dir.path().join("jobs.sqlite");
        let original = fs::read(&db).unwrap();
        let readonly = DownloadRepository::open_readonly(&db).unwrap();
        let first = readonly.history_search(None, "  한글_%'  ").unwrap();
        assert_eq!(first.len(), 128);
        let second = readonly
            .history_search(first.last().map(|row| row.id.as_str()), "한글_%'")
            .unwrap();
        assert_eq!(second.len(), 12);
        assert!(second.iter().all(|row| row.id > first.last().unwrap().id));
        assert_eq!(
            readonly.history_search(None, "legacy-id").unwrap()[0].id,
            "legacy-id"
        );
        assert!(
            readonly
                .history_search(None, "LEGACY_ARGV_SECRET")
                .unwrap()
                .is_empty(),
            "argv snapshot is not source URL"
        );
        assert!(
            readonly
                .history_search(None, "unmatched-literal")
                .unwrap()
                .is_empty()
        );
        assert_eq!(
            readonly
                .history_search(None, "")
                .unwrap()
                .iter()
                .map(|r| &r.id)
                .collect::<Vec<_>>(),
            readonly
                .history(None)
                .unwrap()
                .iter()
                .map(|r| &r.id)
                .collect::<Vec<_>>()
        );
        for query in ["x".repeat(257), "한".repeat(86), "a\0b".into()] {
            assert!(readonly.history_search(None, &query).is_err());
        }
        assert!(
            readonly
                .history_search(Some("invalid/cursor"), "q")
                .is_err()
        );
        assert_eq!(fs::read(&db).unwrap(), original);
    }
    #[test]
    fn history_search_readonly_refuses_bad_options_and_jobs_without_erasing_evidence() {
        let (dir, options, repo) = history_fixture();
        repo.create_job("selected-bad", &options).unwrap();
        let db = dir.path().join("jobs.sqlite");
        for bad in [
            "{".into(),
            "x".repeat(131073),
            r#"{"version":2,"url":"https://example.org/clip"}"#.into(),
            r#"{"version":1,"url":23}"#.into(),
            r#"{"version":1,"url":"https://example.org/clip"}"#.into(),
        ] {
            repo.connection
                .execute(
                    "UPDATE download_jobs SET options_json=?1 WHERE id='selected-bad'",
                    [bad],
                )
                .unwrap();
            let original = fs::read(&db).unwrap();
            assert!(
                DownloadRepository::open_readonly(&db)
                    .unwrap()
                    .history_search(None, "selected-bad")
                    .is_err()
            );
            assert_eq!(fs::read(&db).unwrap(), original);
        }
        repo.connection
            .execute(
                "UPDATE download_jobs SET options_json=NULL,job_json=?1 WHERE id='selected-bad'",
                ["x".repeat(131073)],
            )
            .unwrap();
        let original = fs::read(&db).unwrap();
        assert!(
            DownloadRepository::open_readonly(&db)
                .unwrap()
                .history_search(None, "selected-bad")
                .is_err()
        );
        assert_eq!(fs::read(&db).unwrap(), original);
    }
    #[test]
    fn history_search_missing_corrupt_and_busy_are_readonly_failures() {
        let dir = tempfile::tempdir().unwrap();
        let db = dir.path().join("jobs.sqlite");
        assert!(DownloadRepository::open_readonly(&db).is_err());
        assert!(!db.exists());
        fs::write(&db, b"corrupt sqlite preserve").unwrap();
        assert!(
            DownloadRepository::open_readonly(&db)
                .and_then(|reader| reader.history_search(None, "q"))
                .is_err()
        );
        assert_eq!(fs::read(&db).unwrap(), b"corrupt sqlite preserve");
        fs::remove_file(&db).unwrap();
        let (owned, _options, repo) = history_fixture();
        let db = owned.path().join("jobs.sqlite");
        let original = fs::read(&db).unwrap();
        repo.connection.execute_batch("BEGIN EXCLUSIVE").unwrap();
        let reader = DownloadRepository::open_readonly(&db).unwrap();
        assert!(
            matches!(reader.history_search(None,"fixture"),Err(DownloadError::Database(rusqlite::Error::SqliteFailure(error,_)))if error.code==rusqlite::ErrorCode::DatabaseBusy)
        );
        repo.connection.execute_batch("ROLLBACK").unwrap();
        assert_eq!(fs::read(&db).unwrap(), original);
    }
    #[test]
    fn history_resume_rejects_oversize_corrupt_terminal_and_changed_options() {
        let (_dir, options, repo) = history_fixture();
        assert!(repo.validated_resume("fixture-job").is_err()); // queued
        let mut job = repo.job("fixture-job").unwrap();
        job.state = JobState::Interrupted;
        let valid = serde_json::to_string(&job).unwrap();
        repo.connection
            .execute("UPDATE download_jobs SET job_json=?1", [&valid])
            .unwrap();
        repo.validated_resume("fixture-job").unwrap();
        for invalid in ["{".to_owned(), "x".repeat(131073)] {
            repo.connection
                .execute("UPDATE download_jobs SET job_json=?1", [&invalid])
                .unwrap();
            assert!(repo.validated_resume("fixture-job").is_err());
        }
        repo.connection
            .execute("UPDATE download_jobs SET job_json=?1", [&valid])
            .unwrap();
        let mut changed = options;
        changed.quality = dlpad_core::options::Quality::Height(720);
        repo.connection
            .execute(
                "UPDATE download_jobs SET options_json=?1",
                [changed.to_stored_json().unwrap()],
            )
            .unwrap();
        assert!(repo.validated_resume("fixture-job").is_err());
        assert!(repo.validated_resume(&"x".repeat(81)).is_err());
        assert!(repo.validated_resume("invalid/id").is_err());
        assert!(repo.job("fixture-job").unwrap().attempts.is_empty());
    }
    #[test]
    fn history_bounds_ids_and_latest_version_without_exposing_tokens() {
        let (_dir, _options, repo) = history_fixture();
        let mut job = repo.job("fixture-job").unwrap();
        job.state = JobState::Interrupted;
        job.attempts.push(dlpad_core::jobs::Attempt {
            id: "owned-attempt".into(),
            number: 1,
        });
        repo.connection
            .execute(
                "UPDATE download_jobs SET job_json=?1",
                [serde_json::to_string(&job).unwrap()],
            )
            .unwrap();
        repo.connection.execute("INSERT INTO download_attempts(job_id,attempt_id,number,tool_version) VALUES('fixture-job','owned-attempt',1,'2026.08.19')", []).unwrap();
        assert_eq!(
            repo.history(None).unwrap()[0].tool_version.as_deref(),
            Some("2026.08.19")
        );
        for invalid in [
            "Authorization=secret".to_owned(),
            "9".repeat(49),
            String::new(),
        ] {
            repo.connection
                .execute("UPDATE download_attempts SET tool_version=?1", [invalid])
                .unwrap();
            assert!(repo.history(None).unwrap()[0].tool_version.is_none());
        }
        assert!(repo.history(Some("invalid/id")).is_err());
        repo.connection
            .execute("UPDATE download_jobs SET id=?1", ["x".repeat(8192)])
            .unwrap();
        assert!(repo.history(None).is_err());
    }
    #[test]
    fn history_is_bounded_and_keyset_pages_do_not_repeat() {
        let (_dir, options, repo) = history_fixture();
        for number in 0..140 {
            repo.create_job(&format!("history-{number:03}"), &options)
                .unwrap();
        }
        let first = repo.history(None).unwrap();
        assert_eq!(first.len(), HISTORY_PAGE_SIZE);
        assert!(first.iter().all(|job| !job.resumable));
        let second = repo
            .history(first.last().map(|job| job.id.as_str()))
            .unwrap();
        assert_eq!(second.len(), 13);
        assert!(second.iter().all(|job| job.id > first.last().unwrap().id));
    }
}

#[cfg(test)]
mod readable_policy_tests {
    use super::*;
    fn fixture() -> (tempfile::TempDir, DownloadRepository) {
        let dir = tempfile::tempdir().unwrap();
        let repo = DownloadRepository::open(&dir.path().join("jobs.sqlite")).unwrap();
        repo.connection.execute("INSERT INTO download_jobs(id,revision,job_json,snapshot,options_json) VALUES('job',0,'{}','[]','{}')",[]).unwrap();
        (dir, repo)
    }
    fn options(root: &Path) -> DownloadOptions {
        DownloadOptions::from_stored_json(&serde_json::json!({"version":1,"url":"https://example.org/fixture","output_dir":root,"quality":"Best","time_range":null,"advanced":[],"ffmpeg":null,"deno":null,"subtitles":null}).to_string()).unwrap()
    }
    fn session(cancel: &Arc<AtomicBool>) -> DownloadSession<'_> {
        DownloadSession {
            #[cfg(windows)]
            runtime_environment: None,
            cancel,
            pause: None,
            progress: None,
            cookies: None,
            managed: None,
        }
    }
    #[test]
    fn mismatched_filename_style_is_rejected_before_attempt_or_tool_execution() {
        let dir = tempfile::tempdir().unwrap();
        let repo = DownloadRepository::open(&dir.path().join("jobs.sqlite")).unwrap();
        let options = options(dir.path());
        repo.create_job("actual", &options).unwrap();
        let cancel = Arc::new(AtomicBool::new(false));
        let missing = dir.path().join("never-run-tool");
        assert!(matches!(
            repo.execute_with_session_naming(
                "actual",
                &options,
                &missing,
                session(&cancel),
                Some(FinalNameStyle::TitleWithJobId)
            ),
            Err(DownloadError::Policy(
                "job filename options snapshot changed"
            ))
        ));
        assert!(repo.job("actual").unwrap().attempts.is_empty());
        assert_eq!(repo.job("actual").unwrap().revision, 0);
    }
    #[test]
    fn repository_recovers_exact_human_batch_without_reinterpreting_new_metadata() {
        let dir = tempfile::tempdir().unwrap();
        let root = fs::canonicalize(dir.path()).unwrap();
        let repo = DownloadRepository::open(&root.join("jobs.sqlite")).unwrap();
        let options = options(&root);
        repo.create_job("job", &options).unwrap();
        let partial = root.join(".dlpad-partials/job");
        fs::create_dir_all(&partial).unwrap();
        let sources = ["job.mp4", "job.ko.vtt"].map(|name| partial.join(name));
        for source in &sources {
            fs::write(source, b"synthetic").unwrap();
        }
        let plan =
            publication::Plan::prepare_readable(&root, "job", &sources, "원래 제목 🎬").unwrap();
        let mut job = repo.job("job").unwrap();
        job.begin_attempt("attempt").unwrap();
        job.transition(JobState::Downloading).unwrap();
        job.transition(JobState::Verifying).unwrap();
        repo.save(&job, 0).unwrap();
        let mut json: serde_json::Value =
            serde_json::from_str(&options.to_stored_json().unwrap()).unwrap();
        json["final_name_style"] = "TitleWithJobId".into();
        repo.connection
            .execute(
                "UPDATE download_jobs SET options_json=?1,planned_artifacts_json=?2 WHERE id='job'",
                params![json.to_string(), plan.json().unwrap()],
            )
            .unwrap();
        fs::hard_link(&sources[0], plan.first_target()).unwrap();
        let cancel = Arc::new(AtomicBool::new(false));
        assert!(matches!(
            repo.execute_with_session_naming(
                "job",
                &options,
                &root.join("never-run"),
                session(&cancel),
                Some(FinalNameStyle::TitleWithJobId)
            ),
            Err(DownloadError::Policy(
                "persisted readable-name publication requires recovery"
            ))
        ));
        assert_eq!(repo.recover().unwrap(), 1);
        assert_eq!(repo.recover().unwrap(), 0);
        let complete = repo.job("job").unwrap();
        assert_eq!(complete.state, JobState::Downloaded);
        assert_eq!(complete.artifacts.len(), 2);
        assert!(complete.artifacts.iter().all(|a| {
            a.path
                .file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("원래 제목 🎬 [job].")
        }));
        assert!(!root.join("job.mp4").exists());
    }
    #[test]
    fn naming_policy_is_loaded_from_persistent_job_options_and_bad_values_are_not_legacy() {
        let (_dir, repo) = fixture();
        assert_eq!(repo.persisted_name_style("job").unwrap(), None);
        repo.connection
            .execute(
                "UPDATE download_jobs SET options_json=?1 WHERE id='job'",
                [r#"{"version":1,"final_name_style":"TitleWithJobId"}"#],
            )
            .unwrap();
        assert_eq!(
            repo.persisted_name_style("job").unwrap(),
            Some(FinalNameStyle::TitleWithJobId)
        );
        for invalid in [
            r#"{"final_name_style":"unsafe-user-template"}"#.to_owned(),
            "x".repeat(131073),
        ] {
            repo.connection
                .execute(
                    "UPDATE download_jobs SET options_json=?1 WHERE id='job'",
                    [invalid],
                )
                .unwrap();
            assert!(repo.persisted_name_style("job").is_err());
        }
    }
}
#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use dlpad_core::options::{Quality, SourceUrl};
    use std::{os::unix::fs::PermissionsExt, sync::atomic::Ordering};
    fn fixture(
        mode: &str,
    ) -> (
        tempfile::TempDir,
        PathBuf,
        DownloadOptions,
        DownloadRepository,
    ) {
        let dir = tempfile::tempdir().unwrap();
        let tool = dir.path().join("yt-dlp-fixture");
        let script = format!(
            r#"#!/usr/bin/python3
import sys,json,pathlib
args=sys.argv[1:]
if '--version' in args: print('2026.09.30');sys.exit(0)
mode={mode:?}
if '--dump-single-json' in args:
 if mode=='metadata-noise':
  for _ in range(5000): print('warning '+ 'x'*100,file=sys.stderr)
 padding='x'*(9*1024*1024 if mode=='metadata-overflow' else 100000 if mode in ['metadata-large','metadata-noise'] else 0)
 print(json.dumps({{'id':'fixture','duration':10,'formats':padding,'url':'https://secret.invalid/?token=secret'}}));sys.exit(0)
root=pathlib.Path(args[args.index('--paths')+1])
name=args[args.index('--output')+1].replace('%(ext)s','mp4')
path=root/name
if mode=='flood':
 for _ in range(5000): print('progress: '+ 'x'*100)
if mode=='pause':
 import time
 path.with_suffix('.part').write_bytes(b'paused partial')
 time.sleep(30)
if mode=='nonzero': print('failed',file=sys.stderr);sys.exit(3)
if mode=='escape': path=root.parent.parent/'outside.mp4'
if mode!='missing': path.write_bytes(b'synthetic-media')
if '--write-subs' in args:
 subs={{}}
 for language in ['ko','en']:
  if mode=='subtitle-none' or (language=='en' and '--write-auto-subs' not in args):
   subs[language]={{}};continue
  sidecar=path.with_suffix('.'+language+'.vtt')
  sidecar.write_bytes(b'WEBVTT synthetic '+language.encode())
  subs[language]={{'filepath':str(sidecar),'ext':'vtt'}}
 if mode=='subtitle-bad': subs['ko']['filepath']=str(root.parent/'foreign.vtt')
 print('DLPAD_SUBTITLES:'+json.dumps({{'id':'fixture','requested_subtitles':subs}}))
print('DLPAD_ARTIFACT:'+json.dumps(str(path)))
"#
        );
        fs::write(&tool, script).unwrap();
        fs::set_permissions(&tool, fs::Permissions::from_mode(0o700)).unwrap();
        let options = DownloadOptions {
            url: SourceUrl::parse("https://example.org/fixture").unwrap(),
            output_dir: dir.path().join("media 한글"),
            quality: Quality::Best,
            time_range: None,
            advanced: vec![],
            ffmpeg: None,
            deno: None,
            final_name_style: None,
            subtitles: None,
        };
        let repo = DownloadRepository::open(&dir.path().join("jobs.sqlite")).unwrap();
        repo.create_job("fixture-job", &options).unwrap();
        (dir, tool, options, repo)
    }
    #[test]
    fn oversized_range_has_unknown_processing_eta_without_source_fallback() {
        let range = dlpad_core::options::TimeRange::new(0, u64::MAX, false).unwrap();
        assert_eq!(
            processing_duration_us(Some(range), &serde_json::json!({"duration": 5})),
            None
        );
        let range = dlpad_core::options::TimeRange::new(1000, 5000, false).unwrap();
        assert_eq!(
            processing_duration_us(Some(range), &serde_json::json!({})),
            Some(4_000_000)
        );
    }
    #[test]
    fn running_pause_ack_preserves_partial_and_reuses_job() {
        let (_dir, tool, options, repo) = fixture("pause");
        let cancel = Arc::new(AtomicBool::new(false));
        let pause = Arc::new(AtomicBool::new(false));
        let partial = options
            .output_dir
            .join(".dlpad-partials/fixture-job/fixture-job.part");
        let observer_path = partial.clone();
        let signal_cancel = cancel.clone();
        let signal_pause = pause.clone();
        let requester = std::thread::spawn(move || {
            let start = std::time::Instant::now();
            while !observer_path.is_file() && start.elapsed() < Duration::from_secs(5) {
                std::thread::sleep(Duration::from_millis(10));
            }
            let observed = observer_path.is_file();
            signal_pause.store(true, Ordering::Release);
            signal_cancel.store(true, Ordering::Release);
            observed
        });
        assert!(
            repo.execute_with_control(
                "fixture-job",
                &options,
                &tool,
                &cancel,
                &pause,
                crate::metrics::ProgressFeed::default()
            )
            .is_err()
        );
        assert!(requester.join().unwrap());
        assert_eq!(repo.job("fixture-job").unwrap().state, JobState::Paused);
        assert_eq!(fs::read(&partial).unwrap(), b"paused partial");
        let script = fs::read_to_string(&tool)
            .unwrap()
            .replace("mode=\"pause\"", "mode=\"success\"");
        fs::write(&tool, script).unwrap();
        cancel.store(false, Ordering::Release);
        pause.store(false, Ordering::Release);
        assert_eq!(
            repo.resume_with_control(
                "fixture-job",
                &tool,
                &cancel,
                &pause,
                crate::metrics::ProgressFeed::default()
            )
            .unwrap()
            .job
            .attempts
            .len(),
            2
        );
        assert_eq!(fs::read(&partial).unwrap(), b"paused partial");
    }
    #[test]
    fn pause_requires_cancel_ack_and_keeps_partial_for_resume() {
        let (_dir, tool, options, repo) = fixture("success");
        let cancel = Arc::new(AtomicBool::new(true));
        let pause = Arc::new(AtomicBool::new(true));
        assert!(
            repo.execute_with_control(
                "fixture-job",
                &options,
                &tool,
                &cancel,
                &pause,
                crate::metrics::ProgressFeed::default()
            )
            .is_err()
        );
        assert_eq!(repo.job("fixture-job").unwrap().state, JobState::Paused);
        cancel.store(false, Ordering::Release);
        pause.store(false, Ordering::Release);
        let completed = repo
            .resume_with_control(
                "fixture-job",
                &tool,
                &cancel,
                &pause,
                crate::metrics::ProgressFeed::default(),
            )
            .unwrap();
        assert_eq!(completed.job.state, JobState::Downloaded);
        assert_eq!(completed.job.attempts.len(), 2);
        let (_dir, tool, options, repo) = fixture("nonzero");
        pause.store(true, Ordering::Release);
        assert!(
            repo.execute_with_control(
                "fixture-job",
                &options,
                &tool,
                &cancel,
                &pause,
                crate::metrics::ProgressFeed::default()
            )
            .is_err()
        );
        assert_eq!(repo.job("fixture-job").unwrap().state, JobState::Failed);
    }
    #[test]
    fn resume_keeps_id_adds_attempt_and_rejects_terminal_job() {
        let (_dir, tool, options, repo) = fixture("nonzero");
        let cancel = Arc::new(AtomicBool::new(false));
        assert!(
            repo.execute("fixture-job", &options, &tool, &cancel)
                .is_err()
        );
        let retained = options
            .output_dir
            .join(".dlpad-partials/fixture-job/fixture-job.mp4.part");
        fs::write(&retained, b"retained partial bytes").unwrap();
        let script = fs::read_to_string(&tool)
            .unwrap()
            .replace("mode=\"nonzero\"", "mode=\"success\"")
            .replace("name=args[args.index('--output')+1]", "assert (root/'fixture-job.mp4.part').read_bytes()==b'retained partial bytes'\nname=args[args.index('--output')+1]");
        fs::write(&tool, script).unwrap();
        let result = repo.resume("fixture-job", &tool, &cancel).unwrap();
        assert_eq!(fs::read(&retained).unwrap(), b"retained partial bytes");
        assert_eq!(result.job.id, "fixture-job");
        assert_eq!(result.job.state, JobState::Downloaded);
        assert_eq!(result.job.attempts.len(), 2);
        assert_eq!(result.job.attempts[1].number, 2);
        assert_ne!(result.job.attempts[0].id, result.job.attempts[1].id);
        assert!(repo.resume("fixture-job", &tool, &cancel).is_err());
        assert_eq!(repo.job("fixture-job").unwrap().attempts.len(), 2);
    }
    #[test]
    fn stored_options_fail_closed_without_new_attempt() {
        for value in [None, Some("{}"), Some(" ")] {
            let (_dir, tool, options, repo) = fixture("nonzero");
            let cancel = Arc::new(AtomicBool::new(false));
            assert!(
                repo.execute("fixture-job", &options, &tool, &cancel)
                    .is_err()
            );
            repo.connection
                .execute("UPDATE download_jobs SET options_json=?1", [value])
                .unwrap();
            assert!(repo.resume("fixture-job", &tool, &cancel).is_err());
            assert_eq!(repo.job("fixture-job").unwrap().attempts.len(), 1);
        }
        let (_dir, tool, mut options, repo) = fixture("nonzero");
        let cancel = Arc::new(AtomicBool::new(false));
        assert!(repo.resume("fixture-job", &tool, &cancel).is_err());
        assert!(
            repo.execute("fixture-job", &options, &tool, &cancel)
                .is_err()
        );
        options.quality = Quality::Audio;
        repo.connection
            .execute(
                "UPDATE download_jobs SET options_json=?1",
                [options.to_stored_json().unwrap()],
            )
            .unwrap();
        assert!(repo.resume("fixture-job", &tool, &cancel).is_err());
        assert_eq!(repo.job("fixture-job").unwrap().attempts.len(), 1);
    }
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    #[test]
    fn selected_deno_is_same_literal_runtime_for_metadata_and_download_and_reopens() {
        let (dir, tool, mut options, repo) = fixture("success");
        let folder = dir.path().join("Deno 한글 공백");
        fs::create_dir(&folder).unwrap();
        let deno = folder.join("deno");
        let source = b"#!/usr/bin/python3
import sys
assert sys.argv[1:]==['--version']
print('deno 2.9.7 (stable, release, x86_64-unknown-linux-gnu)')
print('v8 15.0.245.2-rusty')
print('typescript 6.0.3')
";
        fs::write(&deno, source).unwrap();
        fs::set_permissions(&deno, fs::Permissions::from_mode(0o700)).unwrap();
        let trace = dir.path().join("argv.jsonl");
        let script = fs::read_to_string(&tool).unwrap().replace(
            "args=sys.argv[1:]",
            &format!(
                r#"args=sys.argv[1:]
if '--version' not in args:
 with open({},'a') as f: f.write(json.dumps(args)+chr(10))"#,
                serde_json::to_string(&trace).unwrap()
            ),
        );
        fs::write(&tool, script).unwrap();
        options.deno = Some(deno.clone());
        repo.create_job("explicit-deno", &options).unwrap();
        let cancel = Arc::new(AtomicBool::new(false));
        let result = repo
            .execute("explicit-deno", &options, &tool, &cancel)
            .unwrap();
        assert_eq!(result.deno_version.as_deref(), Some("2.9.7"));
        let calls: Vec<Vec<String>> = fs::read_to_string(trace)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        assert_eq!(calls.len(), 2);
        let expected = format!("deno:{}", deno.canonicalize().unwrap().to_str().unwrap());
        for args in calls {
            assert_eq!(
                args.iter().filter(|arg| *arg == "--no-js-runtimes").count(),
                1
            );
            assert_eq!(args.iter().filter(|arg| *arg == "--js-runtimes").count(), 1);
            let index = args.iter().position(|arg| arg == "--js-runtimes").unwrap();
            assert_eq!(args[index + 1], expected);
            assert!(!args.iter().any(|arg| arg == "--remote-components"));
        }
        drop(repo);
        let reopened = DownloadRepository::open(&dir.path().join("jobs.sqlite")).unwrap();
        assert_eq!(
            reopened.deno_version("explicit-deno").unwrap().as_deref(),
            Some("2.9.7")
        );
        let recorded:String=reopened.connection.query_row("SELECT tool_path FROM download_attempt_components WHERE job_id='explicit-deno' AND component='deno'",[],|row|row.get(0)).unwrap();
        assert_eq!(PathBuf::from(recorded), deno.canonicalize().unwrap());
        assert_eq!(fs::read(deno).unwrap(), source);
    }
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    #[test]
    fn invalid_selected_deno_fails_before_metadata_without_runtime_evidence_or_artifact() {
        let (dir, tool, mut options, repo) = fixture("success");
        let deno = dir.path().join("invalid-deno");
        let source = b"#!/usr/bin/python3
print('untrusted banner')
";
        fs::write(&deno, source).unwrap();
        fs::set_permissions(&deno, fs::Permissions::from_mode(0o700)).unwrap();
        options.deno = Some(deno.clone());
        repo.create_job("invalid-deno", &options).unwrap();
        let cancel = Arc::new(AtomicBool::new(false));
        assert!(
            repo.execute("invalid-deno", &options, &tool, &cancel)
                .is_err()
        );
        assert!(repo.deno_version("invalid-deno").unwrap().is_none());
        assert_eq!(repo.job("invalid-deno").unwrap().state, JobState::Failed);
        assert!(repo.job("invalid-deno").unwrap().artifacts.is_empty());
        assert_eq!(fs::read(deno).unwrap(), source);
    }
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    #[test]
    fn cancellation_during_deno_discovery_is_cancelled_without_metadata_or_artifact() {
        let (dir, tool, mut options, repo) = fixture("success");
        let deno = dir.path().join("slow-deno");
        let ready = dir.path().join("deno-ready");
        let code = format!(
            "#!/usr/bin/python3\nimport pathlib,time\npathlib.Path({}).write_text('ready')\ntime.sleep(30)\n",
            serde_json::to_string(&ready).unwrap()
        );
        fs::write(&deno, &code).unwrap();
        fs::set_permissions(&deno, fs::Permissions::from_mode(0o700)).unwrap();
        options.deno = Some(deno.clone());
        repo.create_job("cancel-deno", &options).unwrap();
        let cancel = Arc::new(AtomicBool::new(false));
        let intent = cancel.clone();
        let canceller = std::thread::spawn(move || {
            let deadline = std::time::Instant::now() + Duration::from_secs(3);
            while !ready.exists() && std::time::Instant::now() < deadline {
                std::thread::sleep(Duration::from_millis(10));
            }
            assert!(ready.exists(), "version child must actually start");
            intent.store(true, std::sync::atomic::Ordering::Release);
        });
        let result = repo.execute("cancel-deno", &options, &tool, &cancel);
        canceller.join().unwrap();
        assert!(
            matches!(result,Err(DownloadError::DenoDiscovery(error)) if error.kind()==io::ErrorKind::Interrupted)
        );
        assert_eq!(repo.job("cancel-deno").unwrap().state, JobState::Cancelled);
        assert!(repo.deno_version("cancel-deno").unwrap().is_none());
        assert!(repo.job("cancel-deno").unwrap().artifacts.is_empty());
        assert_eq!(fs::read_to_string(deno).unwrap(), code);
    }
    #[test]
    fn selected_ffmpeg_evidence_survives_reopen_and_snapshot_change_is_rejected() {
        let (dir, tool, mut options, repo) = fixture("success");
        let folder = dir.path().join("FFmpeg 한글 공백");
        fs::create_dir(&folder).unwrap();
        let ffmpeg = folder.join("ffmpeg");
        fs::write(
            &ffmpeg,
            "#!/usr/bin/python3\nprint('ffmpeg version 7.1-test fixture')\n",
        )
        .unwrap();
        fs::set_permissions(&ffmpeg, fs::Permissions::from_mode(0o700)).unwrap();
        options.ffmpeg = Some(ffmpeg.clone());
        repo.create_job("explicit-ffmpeg", &options).unwrap();
        let mut changed = options.clone();
        changed.ffmpeg = None;
        let cancel = Arc::new(AtomicBool::new(false));
        assert!(matches!(
            repo.execute("explicit-ffmpeg", &changed, &tool, &cancel),
            Err(DownloadError::Policy("job options snapshot changed"))
        ));
        assert!(repo.job("explicit-ffmpeg").unwrap().attempts.is_empty());
        let result = repo
            .execute("explicit-ffmpeg", &options, &tool, &cancel)
            .unwrap();
        assert_eq!(result.ffmpeg_version.as_deref(), Some("7.1-test"));
        drop(repo);
        let reopened = DownloadRepository::open(&dir.path().join("jobs.sqlite")).unwrap();
        assert_eq!(
            reopened
                .ffmpeg_version("explicit-ffmpeg")
                .unwrap()
                .as_deref(),
            Some("7.1-test")
        );
        let path: String = reopened
            .connection
            .query_row(
                "SELECT tool_path FROM download_attempt_components WHERE job_id='explicit-ffmpeg'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(PathBuf::from(path), ffmpeg.canonicalize().unwrap());
        assert_eq!(
            fs::read(&ffmpeg).unwrap(),
            b"#!/usr/bin/python3\nprint('ffmpeg version 7.1-test fixture')\n"
        );
    }
    #[test]
    fn cancellation_during_ffmpeg_discovery_stops_without_artifact() {
        let (dir, tool, mut options, repo) = fixture("success");
        let marker = dir.path().join("version-started");
        let ffmpeg = dir.path().join("ffmpeg");
        let encoded = serde_json::to_string(&marker.to_string_lossy()).unwrap();
        fs::write(&ffmpeg, format!("#!/usr/bin/python3\nimport pathlib,time\npathlib.Path({encoded}).write_text('ready')\ntime.sleep(10)\nprint('ffmpeg version 7.1 fixture')\n")).unwrap();
        fs::set_permissions(&ffmpeg, fs::Permissions::from_mode(0o700)).unwrap();
        options.ffmpeg = Some(ffmpeg);
        repo.create_job("cancel-ffmpeg", &options).unwrap();
        let cancel = Arc::new(AtomicBool::new(false));
        let signal = cancel.clone();
        let waiter = std::thread::spawn(move || {
            let deadline = std::time::Instant::now() + Duration::from_secs(5);
            while !marker.exists() && std::time::Instant::now() < deadline {
                std::thread::sleep(Duration::from_millis(20));
            }
            let started = marker.exists();
            signal.store(true, std::sync::atomic::Ordering::Release);
            started
        });
        assert!(
            matches!(repo.execute("cancel-ffmpeg", &options, &tool, &cancel), Err(DownloadError::Execution { stage: "ffmpeg-version", result }) if result.outcome == ProcessOutcome::Cancelled)
        );
        assert!(waiter.join().unwrap());
        assert_eq!(
            repo.job("cancel-ffmpeg").unwrap().state,
            JobState::Cancelled
        );
        assert!(repo.ffmpeg_version("cancel-ffmpeg").unwrap().is_none());
    }
    #[test]
    fn missing_and_malformed_ffmpeg_fail_without_version_or_artifact() {
        for mode in ["missing", "malformed", "directory", "nonzero"] {
            let (dir, tool, mut options, repo) = fixture("success");
            let ffmpeg = dir.path().join("ffmpeg");
            if mode == "directory" {
                fs::create_dir(&ffmpeg).unwrap();
            } else if mode != "missing" {
                let script = if mode == "nonzero" {
                    "#!/usr/bin/python3\nimport sys\nsys.exit(3)\n"
                } else {
                    "#!/usr/bin/python3\nprint('ffmpeg version https://secret.invalid/token fixture')\n"
                };
                fs::write(&ffmpeg, script).unwrap();
                fs::set_permissions(&ffmpeg, fs::Permissions::from_mode(0o700)).unwrap();
            }
            options.ffmpeg = Some(ffmpeg);
            repo.create_job("invalid-ffmpeg", &options).unwrap();
            assert!(
                repo.execute(
                    "invalid-ffmpeg",
                    &options,
                    &tool,
                    &Arc::new(AtomicBool::new(false))
                )
                .is_err()
            );
            let job = repo.job("invalid-ffmpeg").unwrap();
            assert_eq!(job.state, JobState::Failed);
            assert!(job.artifacts.is_empty());
            assert!(repo.ffmpeg_version("invalid-ffmpeg").unwrap().is_none());
        }
    }
    #[test]
    fn startup_recovery_changes_active_job_once_and_keeps_attempts() {
        let (dir, _tool, _options, repo) = fixture("success");
        let mut job = repo.job("fixture-job").unwrap();
        let old = job.revision;
        job.begin_attempt("previous-attempt").unwrap();
        job.transition(JobState::Downloading).unwrap();
        repo.save(&job, old).unwrap();
        drop(repo);
        assert_eq!(recover_at_startup(dir.path()).unwrap(), 1);
        assert_eq!(recover_at_startup(dir.path()).unwrap(), 0);
        let reopened = DownloadRepository::open(&dir.path().join("jobs.sqlite")).unwrap();
        let recovered = reopened.job("fixture-job").unwrap();
        assert_eq!(recovered.state, JobState::Interrupted);
        assert_eq!(recovered.attempts.len(), 1);
        assert_eq!(recovered.attempts[0].id, "previous-attempt");
        assert!(recovered.artifacts.is_empty());
    }
    #[test]
    fn startup_missing_corrupt_and_symlink_journals_preserve_files() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(recover_at_startup(&dir.path().join("absent")).unwrap(), 0);
        assert!(!dir.path().join("absent").exists());
        let path = dir.path().join("jobs.sqlite");
        fs::write(&path, b"synthetic corrupt journal").unwrap();
        assert!(recover_at_startup(dir.path()).is_err());
        assert_eq!(fs::read(&path).unwrap(), b"synthetic corrupt journal");
        fs::remove_file(&path).unwrap();
        let target = dir.path().join("external-data");
        fs::write(&target, b"synthetic external data").unwrap();
        std::os::unix::fs::symlink(&target, &path).unwrap();
        assert!(recover_at_startup(dir.path()).is_err());
        assert_eq!(fs::read(&target).unwrap(), b"synthetic external data");
    }
    #[test]
    fn verifies_and_persists_artifact() {
        let (_dir, tool, options, repo) = fixture("success");
        let result = repo
            .execute(
                "fixture-job",
                &options,
                &tool,
                &Arc::new(AtomicBool::new(false)),
            )
            .unwrap();
        assert_eq!(result.job.state, JobState::Downloaded);
        assert_eq!(
            repo.tool_version("fixture-job").unwrap().as_deref(),
            Some("2026.09.30")
        );
        assert_eq!(repo.job("fixture-job").unwrap().artifacts[0].bytes, 15);
        assert!(options.output_dir.join("fixture-job.mp4").is_file());
    }
    #[test]
    fn final_marker_survives_log_flood() {
        let (_dir, tool, options, repo) = fixture("flood");
        let result = repo
            .execute(
                "fixture-job",
                &options,
                &tool,
                &Arc::new(AtomicBool::new(false)),
            )
            .unwrap();
        assert_eq!(result.job.state, JobState::Downloaded);
        assert!(result.execution.truncated);
        assert!(result.execution.logs.len() <= process::MAX_LOG_BYTES);
    }
    #[test]
    fn failed_results_never_complete() {
        for mode in ["nonzero", "missing", "escape"] {
            let (_dir, tool, options, repo) = fixture(mode);
            assert!(
                repo.execute(
                    "fixture-job",
                    &options,
                    &tool,
                    &Arc::new(AtomicBool::new(false))
                )
                .is_err()
            );
            assert_eq!(repo.job("fixture-job").unwrap().state, JobState::Failed);
        }
    }
    #[test]
    fn publication_never_overwrites() {
        let (_dir, tool, options, repo) = fixture("success");
        fs::create_dir_all(&options.output_dir).unwrap();
        let existing = options.output_dir.join("fixture-job.mp4");
        fs::write(&existing, b"existing-user-file").unwrap();
        assert!(
            repo.execute(
                "fixture-job",
                &options,
                &tool,
                &Arc::new(AtomicBool::new(false))
            )
            .is_err()
        );
        assert_eq!(fs::read(existing).unwrap(), b"existing-user-file");
    }
    #[test]
    fn cancel_and_recovery() {
        let (_dir, tool, options, repo) = fixture("success");
        let cancel = Arc::new(AtomicBool::new(false));
        cancel.store(true, Ordering::Release);
        assert!(
            repo.execute("fixture-job", &options, &tool, &cancel)
                .is_err()
        );
        assert_eq!(repo.job("fixture-job").unwrap().state, JobState::Cancelled);
        repo.create_job("recover-job", &options).unwrap();
        let mut job = repo.job("recover-job").unwrap();
        job.begin_attempt("interrupted").unwrap();
        repo.save(&job, 0).unwrap();
        assert_eq!(repo.recover().unwrap(), 1);
        assert_eq!(repo.recover().unwrap(), 0);
        assert_eq!(
            repo.job("recover-job").unwrap().state,
            JobState::Interrupted
        );
    }
    #[test]
    fn large_metadata_and_stderr_are_separate() {
        for mode in ["metadata-large", "metadata-noise"] {
            let (_dir, tool, options, repo) = fixture(mode);
            let result = repo
                .execute(
                    "fixture-job",
                    &options,
                    &tool,
                    &Arc::new(AtomicBool::new(false)),
                )
                .unwrap();
            assert_eq!(result.metadata["id"], "fixture");
            assert!(result.metadata.get("formats").is_none());
            assert!(result.metadata.get("url").is_none());
        }
    }
    #[test]
    fn metadata_cap_fails_closed() {
        let (_dir, tool, options, repo) = fixture("metadata-overflow");
        assert!(
            matches!(repo.execute("fixture-job",&options,&tool,&Arc::new(AtomicBool::new(false))),Err(DownloadError::Execution{stage:"metadata",result}) if result.outcome==ProcessOutcome::OutputLimit)
        );
        assert_eq!(repo.job("fixture-job").unwrap().state, JobState::Failed);
    }
    #[test]
    fn suspicious_version_is_not_persisted() {
        let (_dir, tool, options, repo) = fixture("success");
        let script = fs::read_to_string(&tool)
            .unwrap()
            .replace("2026.09.30", "secret=token");
        fs::write(&tool, script).unwrap();
        assert!(
            repo.execute(
                "fixture-job",
                &options,
                &tool,
                &Arc::new(AtomicBool::new(false))
            )
            .is_err()
        );
        assert_eq!(repo.tool_version("fixture-job").unwrap(), None);
    }
    #[test]
    fn publication_crash_recovery_is_idempotent() {
        let (dir, _tool, options, repo) = fixture("success");
        fs::create_dir_all(&options.output_dir).unwrap();
        let root = fs::canonicalize(&options.output_dir).unwrap();
        let staging = root.join(".dlpad-partials/fixture-job");
        fs::create_dir_all(&staging).unwrap();
        let source = staging.join("fixture-job.mp4");
        let target = root.join("fixture-job.mp4");
        fs::write(&source, b"fixture").unwrap();
        let mut job = repo.job("fixture-job").unwrap();
        job.begin_attempt("crash").unwrap();
        job.transition(JobState::Downloading).unwrap();
        job.transition(JobState::Verifying).unwrap();
        repo.save(&job, 0).unwrap();
        repo.connection.execute("UPDATE download_jobs SET planned_root=?1,planned_path=?2,planned_source=?3,planned_bytes=7 WHERE id='fixture-job'",params![root.to_str(),target.to_str(),source.to_str()]).unwrap();
        fs::hard_link(&source, &target).unwrap();
        drop(repo);
        let repo = DownloadRepository::open(&dir.path().join("jobs.sqlite")).unwrap();
        assert_eq!(repo.recover().unwrap(), 1);
        assert_eq!(repo.recover().unwrap(), 0);
        let job = repo.job("fixture-job").unwrap();
        assert_eq!(job.state, JobState::Downloaded);
        assert_eq!(job.artifacts.len(), 1);
    }
    #[test]
    fn batch_journal_recovers_partial_publication_and_rejects_missing_or_corrupt_evidence() {
        for corrupt in [false, true] {
            let (dir, _tool, options, repo) = fixture("success");
            fs::create_dir_all(&options.output_dir).unwrap();
            let root = fs::canonicalize(&options.output_dir).unwrap();
            let staging = root.join(".dlpad-partials/fixture-job");
            fs::create_dir_all(&staging).unwrap();
            let sources: Vec<_> = [
                "fixture-job.mp4",
                "fixture-job.ko.vtt",
                "fixture-job.en.vtt",
            ]
            .iter()
            .map(|name| staging.join(name))
            .collect();
            for path in &sources {
                fs::write(path, b"fixture").unwrap();
            }
            let plan = publication::Plan::prepare(&root, "fixture-job", &sources).unwrap();
            let mut job = repo.job("fixture-job").unwrap();
            job.begin_attempt("crash").unwrap();
            job.transition(JobState::Downloading).unwrap();
            job.transition(JobState::Verifying).unwrap();
            repo.save(&job, 0).unwrap();
            let json = if corrupt {
                "{}".to_owned()
            } else {
                plan.json().unwrap()
            };
            repo.connection
                .execute(
                    "UPDATE download_jobs SET planned_artifacts_json=?1 WHERE id='fixture-job'",
                    [json],
                )
                .unwrap();
            fs::hard_link(&sources[0], root.join("fixture-job.mp4")).unwrap();
            drop(repo);
            let repo = DownloadRepository::open(&dir.path().join("jobs.sqlite")).unwrap();
            assert_eq!(repo.recover().unwrap(), 1);
            assert_eq!(repo.recover().unwrap(), 0);
            let recovered = repo.job("fixture-job").unwrap();
            if corrupt {
                assert_eq!(recovered.state, JobState::Interrupted);
                assert!(recovered.artifacts.is_empty());
                assert!(!root.join("fixture-job.ko.vtt").exists());
                assert!(sources[1].exists());
            } else {
                assert_eq!(recovered.state, JobState::Downloaded);
                assert_eq!(recovered.artifacts.len(), 3);
                assert_eq!(
                    fs::read(root.join("fixture-job.ko.vtt")).unwrap(),
                    b"fixture"
                );
                assert!(!sources[1].exists());
            }
        }
    }
    #[test]
    fn recovery_rejects_foreign_file_and_migrates_v1() {
        let (dir, _tool, options, repo) = fixture("success");
        fs::create_dir_all(&options.output_dir).unwrap();
        let root = fs::canonicalize(&options.output_dir).unwrap();
        let staging = root.join(".dlpad-partials/fixture-job");
        fs::create_dir_all(&staging).unwrap();
        let source = staging.join("fixture-job.mp4");
        let target = root.join("fixture-job.mp4");
        fs::write(&source, b"fixture").unwrap();
        fs::write(&target, b"foreign").unwrap();
        let mut job = repo.job("fixture-job").unwrap();
        job.begin_attempt("crash").unwrap();
        job.transition(JobState::Downloading).unwrap();
        job.transition(JobState::Verifying).unwrap();
        repo.save(&job, 0).unwrap();
        repo.connection.execute("UPDATE download_jobs SET planned_root=?1,planned_path=?2,planned_source=?3,planned_bytes=7 WHERE id='fixture-job'",params![root.to_str(),target.to_str(),source.to_str()]).unwrap();
        assert_eq!(repo.recover().unwrap(), 1);
        assert_eq!(
            repo.job("fixture-job").unwrap().state,
            JobState::Interrupted
        );
        assert_eq!(fs::read(target).unwrap(), b"foreign");
        let old_path = dir.path().join("v1.sqlite");
        let c = Connection::open(&old_path).unwrap();
        c.execute_batch("CREATE TABLE download_jobs(id TEXT PRIMARY KEY,revision INTEGER NOT NULL,job_json TEXT NOT NULL,snapshot TEXT NOT NULL,tool_path TEXT,tool_version TEXT)").unwrap();
        drop(c);
        assert!(DownloadRepository::open(&old_path).is_ok());
        assert!(DownloadRepository::open(&old_path).is_ok());
    }
    #[test]
    fn subtitle_results_publish_as_one_batch_and_invalid_evidence_never_completes() {
        use dlpad_core::options::{SubtitleLanguage, SubtitleOptions};
        for mode in ["success", "subtitle-none", "subtitle-bad"] {
            let (_dir, tool, mut options, repo) = fixture(mode);
            options.subtitles = Some(SubtitleOptions {
                language: SubtitleLanguage::KoreanAndEnglish,
                auto_generated: true,
            });
            repo.create_job("subtitle-job", &options).unwrap();
            let result = repo.execute(
                "subtitle-job",
                &options,
                &tool,
                &Arc::new(AtomicBool::new(false)),
            );
            let job = repo.job("subtitle-job").unwrap();
            if mode == "subtitle-bad" {
                assert!(result.is_err());
                assert_ne!(job.state, JobState::Downloaded);
                assert!(job.artifacts.is_empty());
                assert!(!options.output_dir.join("subtitle-job.mp4").exists());
            } else {
                result.unwrap();
                assert_eq!(job.state, JobState::Downloaded);
                assert_eq!(
                    job.artifacts.len(),
                    if mode == "subtitle-none" { 1 } else { 3 }
                );
                for artifact in &job.artifacts {
                    assert!(artifact.path.is_file());
                }
                if mode == "success" {
                    assert_eq!(
                        fs::read(options.output_dir.join("subtitle-job.ko.vtt")).unwrap(),
                        b"WEBVTT synthetic ko"
                    );
                    assert_eq!(
                        fs::read(options.output_dir.join("subtitle-job.en.vtt")).unwrap(),
                        b"WEBVTT synthetic en"
                    );
                }
            }
        }
    }
    #[test]
    fn snapshot_and_known_duration() {
        let (_dir, tool, mut options, repo) = fixture("success");
        options.quality = Quality::Audio;
        assert!(
            repo.execute(
                "fixture-job",
                &options,
                &tool,
                &Arc::new(AtomicBool::new(false))
            )
            .is_err()
        );
        assert_eq!(repo.job("fixture-job").unwrap().state, JobState::Queued);
        options.time_range = Some(dlpad_core::options::TimeRange::new(0, 11000, false).unwrap());
        repo.create_job("range-job", &options).unwrap();
        assert!(
            repo.execute(
                "range-job",
                &options,
                &tool,
                &Arc::new(AtomicBool::new(false))
            )
            .is_err()
        );
        assert_eq!(repo.job("range-job").unwrap().state, JobState::Failed);
    }
}

#[cfg(all(test, windows))]
mod windows_recovery_tests {
    use super::*;
    #[test]
    fn legacy_single_plan_requires_native_link_identity() {
        for foreign in [false, true] {
            let directory = tempfile::tempdir().unwrap();
            let root = fs::canonicalize(directory.path()).unwrap();
            let staging = root.join(".dlpad-partials/legacy");
            fs::create_dir_all(&staging).unwrap();
            let source = staging.join("legacy.mp4");
            let target = root.join("legacy.mp4");
            fs::write(&source, b"fixture").unwrap();
            if foreign {
                fs::write(&target, b"foreign").unwrap();
            } else {
                fs::hard_link(&source, &target).unwrap();
            }
            let options = DownloadOptions {
                url: dlpad_core::options::SourceUrl::parse("https://example.org/synthetic")
                    .unwrap(),
                output_dir: root.clone(),
                quality: dlpad_core::options::Quality::Best,
                time_range: None,
                advanced: vec![],
                ffmpeg: None,
                deno: None,
                final_name_style: None,
                subtitles: None,
            };
            let db = directory.path().join("jobs.sqlite");
            let repo = DownloadRepository::open(&db).unwrap();
            let mut job = repo.create_job("legacy", &options).unwrap();
            job.begin_attempt("attempt").unwrap();
            job.transition(JobState::Downloading).unwrap();
            job.transition(JobState::Verifying).unwrap();
            repo.save(&job, 0).unwrap();
            repo.connection.execute("UPDATE download_jobs SET planned_root=?1,planned_path=?2,planned_source=?3,planned_bytes=7 WHERE id='legacy'",params![root.to_str(),target.to_str(),source.to_str()]).unwrap();
            drop(repo);
            let repo = DownloadRepository::open(&db).unwrap();
            assert_eq!(repo.recover().unwrap(), 1);
            assert_eq!(repo.recover().unwrap(), 0);
            let job = repo.job("legacy").unwrap();
            assert_eq!(
                job.state,
                if foreign {
                    JobState::Interrupted
                } else {
                    JobState::Downloaded
                }
            );
            assert_eq!(job.artifacts.len(), if foreign { 0 } else { 1 });
            assert_eq!(
                fs::read(&target).unwrap(),
                if foreign { b"foreign" } else { b"fixture" }
            );
        }
    }
}

#[cfg(test)]
mod session_omission_tests {
    use super::*;

    #[test]
    fn typed_cleanup_uncertainty_survives_authenticated_omission() {
        for marker in [
            process::OwnershipCleanupError::StartupDeferred,
            process::OwnershipCleanupError::StartupUnconfirmed,
            process::OwnershipCleanupError::OutputDrainInterrupted,
            process::OwnershipCleanupError::TerminalUnconfirmed,
        ] {
            let error = DownloadError::from(io::Error::other(marker));
            assert!(matches!(error, DownloadError::SessionCleanupDeferred));
            let result = omit_session_result(Err(error));
            assert!(matches!(result, Err(DownloadError::SessionCleanupDeferred)));
            let error = result.err().unwrap();
            assert!(!format!("{error:?} {error}").contains("secret-cookie"));
        }
        let spoof = DownloadError::from(io::Error::other(
            "guardian startup failed; child cleanup deferred",
        ));
        assert!(matches!(spoof, DownloadError::Io(_)));
    }

    #[test]
    fn authenticated_execution_omission_preserves_control_evidence() {
        for outcome in [
            ProcessOutcome::Exited,
            ProcessOutcome::Cancelled,
            ProcessOutcome::TimedOut,
            ProcessOutcome::OutputLimit,
        ] {
            let result = omit_session_result(Err(DownloadError::Execution {
                stage: "metadata",
                result: ExecutionResult {
                    exit_code: Some(37),
                    logs: "secret-cookie /private/session/path signed-url".into(),
                    truncated: true,
                    outcome,
                },
            }));
            match result {
                Err(DownloadError::Execution { stage, result }) => {
                    assert_eq!(stage, "metadata");
                    assert_eq!(result.logs, SESSION_OUTPUT_OMITTED);
                    assert_eq!(result.outcome, outcome);
                    assert_eq!(result.exit_code, Some(37));
                    assert!(result.truncated);
                }
                _ => panic!("execution failure classification changed"),
            }
        }
    }

    #[test]
    fn authenticated_parser_and_io_error_text_is_not_exposed() {
        let parser = serde_json::from_str::<u8>("\"secret-cookie\"").unwrap_err();
        assert!(parser.to_string().contains("secret-cookie"));
        let cases = [
            DownloadError::Json(parser),
            DownloadError::Io(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "secret-cookie /private/session/path",
            )),
        ];
        for error in cases {
            let result = omit_session_result(Err(error));
            let error = result.err().unwrap();
            assert!(!format!("{error:?} {error}").contains("secret-cookie"));
            assert!(!error.to_string().contains("/private/session/path"));
        }
    }
}

// Append inside download.rs as a separate cfg(test) module: no production seam needed.
#[cfg(test)]
mod sqlite_full_regressions {
    use super::*;
    use dlpad_core::options::{Quality, SourceUrl};

    #[test]
    fn public_create_job_sqlite_full_preserves_prior_journal_and_reopens() {
        let temp = tempfile::tempdir().unwrap();
        let db = temp.path().join("jobs.sqlite");
        let artifact = temp.path().join("original-synthetic.mp4");
        let sentinel = temp.path().join("foreign-sentinel");
        std::fs::write(&artifact, b"owned synthetic prior artifact").unwrap();
        std::fs::write(&sentinel, b"preserve").unwrap();
        let options = DownloadOptions {
            url: SourceUrl::parse("https://fixture.invalid/original").unwrap(),
            output_dir: temp.path().to_path_buf(),
            quality: Quality::Best,
            time_range: None,
            advanced: vec![],
            ffmpeg: None,
            deno: None,
            final_name_style: None,
            subtitles: None,
        };
        let repo = DownloadRepository::open(&db).unwrap();
        repo.create_job("original", &options).unwrap();
        let mut prior = repo.job("original").unwrap();
        prior.begin_attempt("original-1").unwrap();
        repo.save(&prior, 0).unwrap();
        repo.connection.execute("INSERT INTO download_attempts(job_id,attempt_id,number,tool_version) VALUES('original','original-1',1,'2026.10.02')", []).unwrap();
        let prior_attempt: (String, i64, String) = repo.connection.query_row("SELECT attempt_id,number,tool_version FROM download_attempts WHERE job_id='original'", [], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?))).unwrap();
        let original: String = repo
            .connection
            .query_row(
                "SELECT job_json FROM download_jobs WHERE id='original'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        let pages: i64 = repo
            .connection
            .query_row("PRAGMA page_count", [], |r| r.get(0))
            .unwrap();
        let actual: i64 = repo
            .connection
            .query_row(&format!("PRAGMA max_page_count={}", pages + 8), [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(actual, pages + 8);
        let mut full = false;
        let mut failed_id = None;
        let mut inserted = 0;
        for i in 0..512 {
            match repo.create_job(&format!("fill-{i}"), &options) {
                Ok(_) => inserted += 1,
                Err(DownloadError::Database(error)) => {
                    assert_eq!(
                        error.sqlite_error_code(),
                        Some(rusqlite::ErrorCode::DiskFull)
                    );
                    failed_id = Some(format!("fill-{i}"));
                    full = true;
                    break;
                }
                Err(_) => panic!("unexpected non-SQLite failure"),
            }
        }
        assert!(full, "bounded public writes must reach real SQLITE_FULL");
        let failed_id = failed_id.unwrap();
        let partial_rows: i64 = repo
            .connection
            .query_row(
                "SELECT count(*) FROM download_jobs WHERE id=?1",
                [&failed_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(partial_rows, 0);
        let total_rows: i64 = repo
            .connection
            .query_row("SELECT count(*) FROM download_jobs", [], |r| r.get(0))
            .unwrap();
        assert_eq!(total_rows, inserted + 1);
        let after: String = repo
            .connection
            .query_row(
                "SELECT job_json FROM download_jobs WHERE id='original'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(after, original);
        drop(repo);
        let reopened = DownloadRepository::open(&db).unwrap();
        // Read-only state comparison before recovery: active journal recovery intentionally changes state.
        assert_eq!(
            serde_json::to_string(&reopened.job("original").unwrap()).unwrap(),
            original
        );
        let reopened_attempt: (String, i64, String) = reopened.connection.query_row("SELECT attempt_id,number,tool_version FROM download_attempts WHERE job_id='original'", [], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?))).unwrap();
        assert_eq!(reopened_attempt, prior_attempt);
        let integrity: String = reopened
            .connection
            .query_row("PRAGMA integrity_check", [], |r| r.get(0))
            .unwrap();
        assert_eq!(integrity, "ok");
        assert_eq!(reopened.recover().unwrap(), 1);
        assert_eq!(
            reopened.job("original").unwrap().state,
            JobState::Interrupted
        );
        assert_eq!(reopened.recover().unwrap(), 0);
        assert!(reopened.job(&failed_id).is_err());
        let queued: i64 = reopened.connection.query_row(
            "SELECT count(*) FROM download_jobs WHERE id LIKE 'fill-%' AND json_extract(job_json,'$.state')='Queued'", [], |r| r.get(0)
        ).unwrap();
        assert_eq!(queued, inserted);
        assert_eq!(
            std::fs::read(artifact).unwrap(),
            b"owned synthetic prior artifact"
        );
        assert_eq!(std::fs::read(sentinel).unwrap(), b"preserve");
    }
}

// Place inside download.rs as a separate cfg(test) module; no production hook.
#[cfg(test)]
mod sqlite_full_public_recovery {
    use super::*;
    use dlpad_core::options::{Quality, SourceUrl};

    #[test]
    fn published_batch_sqlite_full_preserves_journal_then_recovers_exactly_once() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("media");
        std::fs::create_dir(&root).unwrap();
        let root = std::fs::canonicalize(root).unwrap();
        let stage = root.join(".dlpad-partials/full-recovery");
        std::fs::create_dir_all(&stage).unwrap();
        let source = stage.join("full-recovery.mp4");
        let bytes = b"owned synthetic published artifact";
        std::fs::write(&source, bytes).unwrap();
        let sentinel = root.join("foreign-sentinel");
        std::fs::write(&sentinel, b"preserve").unwrap();
        let db = temp.path().join("jobs.sqlite");
        let repo = DownloadRepository::open(&db).unwrap();
        let options = DownloadOptions {
            url: SourceUrl::parse("https://fixture.invalid/original").unwrap(),
            output_dir: root.clone(),
            quality: Quality::Best,
            time_range: None,
            advanced: vec![],
            ffmpeg: None,
            deno: None,
            final_name_style: None,
            subtitles: None,
        };
        repo.create_job("full-recovery", &options).unwrap();
        repo.create_job("untouched", &options).unwrap();
        let mut job = repo.job("full-recovery").unwrap();
        job.begin_attempt("full-recovery-1").unwrap();
        job.transition(JobState::Downloading).unwrap();
        job.transition(JobState::Verifying).unwrap();
        repo.save(&job, 0).unwrap();
        repo.connection.execute("INSERT INTO download_attempts(job_id,attempt_id,number,tool_version) VALUES('full-recovery','full-recovery-1',1,'2026.10.02')", []).unwrap();
        let plan =
            publication::Plan::prepare(&root, "full-recovery", std::slice::from_ref(&source))
                .unwrap();
        let planned = plan.json().unwrap();
        repo.connection
            .execute(
                "UPDATE download_jobs SET planned_artifacts_json=?1 WHERE id='full-recovery'",
                [&planned],
            )
            .unwrap();
        plan.publish("full-recovery", false).unwrap();
        let final_path = root.join("full-recovery.mp4");
        let raw_before: String = repo
            .connection
            .query_row(
                "SELECT job_json FROM download_jobs WHERE id='full-recovery'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        let other_before = serde_json::to_string(&repo.job("untouched").unwrap()).unwrap();
        repo.connection.execute_batch("CREATE TABLE full_probe(payload BLOB); CREATE TRIGGER full_completion BEFORE UPDATE OF job_json ON download_jobs WHEN NEW.id='full-recovery' AND instr(NEW.job_json,'\"state\":\"Downloaded\"')>0 BEGIN INSERT INTO full_probe VALUES(zeroblob(65536)); END;").unwrap();
        let pages: i64 = repo
            .connection
            .query_row("PRAGMA page_count", [], |r| r.get(0))
            .unwrap();
        let cap: i64 = repo
            .connection
            .query_row(&format!("PRAGMA max_page_count={pages}"), [], |r| r.get(0))
            .unwrap();
        assert_eq!(cap, pages);
        let failure = repo
            .recover()
            .expect_err("completion must encounter actual SQLite page quota");
        let DownloadError::Database(error) = failure else {
            panic!("unexpected recovery failure");
        };
        assert_eq!(
            error.sqlite_error_code(),
            Some(rusqlite::ErrorCode::DiskFull)
        );
        let (raw_after, plan_after): (String, String) = repo.connection.query_row("SELECT job_json,planned_artifacts_json FROM download_jobs WHERE id='full-recovery'", [], |r| Ok((r.get(0)?, r.get(1)?))).unwrap();
        assert_eq!(raw_after, raw_before);
        assert_eq!(plan_after, planned);
        assert_eq!(
            repo.job("full-recovery").unwrap().state,
            JobState::Verifying
        );
        assert_eq!(
            serde_json::to_string(&repo.job("untouched").unwrap()).unwrap(),
            other_before
        );
        let attempts: i64 = repo.connection.query_row("SELECT count(*) FROM download_attempts WHERE job_id='full-recovery' AND attempt_id='full-recovery-1' AND tool_version='2026.10.02'", [], |r| r.get(0)).unwrap();
        assert_eq!(attempts, 1);
        assert_eq!(std::fs::read(&source).unwrap(), bytes);
        assert_eq!(std::fs::read(&final_path).unwrap(), bytes);
        assert_eq!(
            repo.connection
                .query_row("SELECT count(*) FROM full_probe", [], |r| r
                    .get::<_, i64>(0))
                .unwrap(),
            0
        );
        repo.connection.execute_batch("PRAGMA max_page_count=1073741823; DROP TRIGGER full_completion; DROP TABLE full_probe;").unwrap();
        assert_eq!(repo.recover().unwrap(), 1);
        assert_eq!(repo.recover().unwrap(), 0);
        assert!(!source.exists());
        drop(repo);
        let reopened = DownloadRepository::open(&db).unwrap();
        let done = reopened.job("full-recovery").unwrap();
        assert_eq!(done.state, JobState::Downloaded);
        assert_eq!(done.attempts.len(), 1);
        assert_eq!(done.artifacts.len(), 1);
        assert_eq!(reopened.recover().unwrap(), 0);
        assert_eq!(std::fs::read(final_path).unwrap(), bytes);
        assert_eq!(std::fs::read(sentinel).unwrap(), b"preserve");
        assert_eq!(
            reopened
                .connection
                .query_row("PRAGMA integrity_check", [], |r| r.get::<_, String>(0))
                .unwrap(),
            "ok"
        );
    }
}
