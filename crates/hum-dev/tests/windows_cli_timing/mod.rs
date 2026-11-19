use std::{
    cell::RefCell,
    fs::{self, File},
    io::{self, Write},
    path::PathBuf,
    sync::OnceLock,
    time::Instant,
};

const MAX_ROWS: usize = 256;
const MAX_BYTES: usize = 32 * 1024;
static EPOCH: OnceLock<Instant> = OnceLock::new();

thread_local! {
    static ACTIVE: RefCell<Option<Record>> = const { RefCell::new(None) };
}

struct Record {
    name: &'static str,
    directory: PathBuf,
    start: u128,
    rows: Vec<(&'static str, u128, u128)>,
    saturated: bool,
}

fn now() -> u128 {
    EPOCH.get_or_init(Instant::now).elapsed().as_nanos()
}

pub(super) struct Case(bool);

pub(super) fn case(name: &'static str) -> Case {
    if !cfg!(windows) {
        return Case(false);
    }
    let Some(directory) = std::env::var_os("HUM_WINDOWS_CLI_TIMING_DIR").map(PathBuf::from) else {
        return Case(false);
    };
    if !directory.is_absolute() {
        return Case(false);
    }
    Case(
        ACTIVE
            .try_with(|cell| {
                let Ok(mut slot) = cell.try_borrow_mut() else {
                    return false;
                };
                if slot.is_some() {
                    return false;
                }
                *slot = Some(Record {
                    name,
                    directory,
                    start: now(),
                    rows: Vec::with_capacity(MAX_ROWS),
                    saturated: false,
                });
                true
            })
            .unwrap_or(false),
    )
}

pub(super) struct Span(&'static str, Option<u128>);

pub(super) fn span(name: &'static str) -> Span {
    let start = ACTIVE
        .try_with(|cell| {
            cell.try_borrow()
                .ok()
                .filter(|slot| slot.is_some())
                .map(|_| now())
        })
        .ok()
        .flatten();
    Span(name, start)
}

impl Drop for Span {
    fn drop(&mut self) {
        if let Some(start) = self.1 {
            let end = now();
            let _ = ACTIVE.try_with(|cell| {
                if let Ok(mut slot) = cell.try_borrow_mut()
                    && let Some(record) = slot.as_mut()
                {
                    if record.rows.len() < MAX_ROWS {
                        record.rows.push((self.0, start, end));
                    } else {
                        record.saturated = true;
                    }
                }
            });
        }
    }
}

impl Drop for Case {
    fn drop(&mut self) {
        if self.0 {
            let end = now();
            let record = ACTIVE
                .try_with(|cell| cell.try_borrow_mut().ok().and_then(|mut slot| slot.take()))
                .ok()
                .flatten();
            if let Some(record) = record {
                // Flush after fixture and lock guards have dropped. No test uses this result.
                let _ = record.publish(end, std::thread::panicking());
            }
        }
    }
}

impl Record {
    fn publish(self, end: u128, unwound: bool) -> io::Result<()> {
        let pid = std::process::id();
        let mut bytes = Vec::new();
        writeln!(
            bytes,
            "hum.windows-cli-timing.v1\t{pid}\t{}\tns\t{}",
            self.name, self.start
        )?;
        for (name, start, end) in &self.rows {
            writeln!(bytes, "span\t{name}\t{start}\t{end}")?;
        }
        writeln!(
            bytes,
            "end\t{end}\t{}\t{}\t{unwound}",
            self.rows.len(),
            self.saturated
        )?;
        if bytes.len() > MAX_BYTES {
            return Err(io::Error::other("timing receipt byte limit"));
        }
        let partial = self.directory.join(format!("{pid}-{}.partial", self.name));
        let complete = partial.with_extension("tsv");
        let mut file = File::create_new(&partial)?;
        if complete.try_exists()? {
            return Err(io::Error::other("timing receipt already published"));
        }
        file.write_all(&bytes)?;
        file.flush()?;
        drop(file);
        fs::rename(partial, complete)
    }
}
