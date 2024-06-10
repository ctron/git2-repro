use anyhow::Error;
use git2::build::RepoBuilder;
use git2::{ErrorClass, ErrorCode, FetchOptions, RemoteCallbacks, Repository, ResetType};
use std::collections::HashSet;
use std::path::PathBuf;
use tracing::info_span;

const SOURCE: &str = "https://github.com/CVEProject/cvelistV5.git";
// const PATH: &str = "/home/jreimann/git/git2-repro/https%3A%2F%2Fgithub.com%2FCVEProject%2FcvelistV5";
const PATH: &str = "cvelistV5";

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let mut args = std::env::args();

    let path = args.next().expect("requires path");
    let source = args.next().unwrap_or_else(|| SOURCE.into());

    let path = &PathBuf::from(PATH);
    let continuation: Option<&str> = None;

    log::debug!("Starting run for: {}", SOURCE);

    log::info!("Cloning {} into {}", SOURCE, path.display());

    let mut cb = RemoteCallbacks::new();
    cb.transfer_progress(|progress| {
        let received = progress.received_objects();
        let total = progress.total_objects();
        let bytes = progress.received_bytes();

        log::trace!("Progress - objects: {received} of {total}, bytes: {bytes}");

        true
    });
    cb.update_tips(|refname, a, b| {
        if a.is_zero() {
            log::debug!("[new]     {:20} {}", b, refname);
        } else {
            log::debug!("[updated] {:10}..{:10} {}", a, b, refname);
        }
        true
    });

    let mut fo = FetchOptions::new();
    fo.remote_callbacks(cb);

    // clone or open repository

    let result = info_span!("clone repository")
        .in_scope(|| RepoBuilder::new().fetch_options(fo).clone(SOURCE, path));

    let repo = match result {
        Ok(repo) => repo,
        Err(err) if err.code() == ErrorCode::Exists && err.class() == ErrorClass::Invalid => {
            log::info!("Already exists, opening ...");
            let repo = info_span!("open repository").in_scope(|| Repository::open(path))?;

            info_span!("fetching updates").in_scope(|| {
                log::debug!("Fetching updates");
                let mut remote = repo.find_remote("origin")?;
                remote.fetch(&[] as &[&str], None, None)?;
                log::debug!("Disconnecting");
                remote.disconnect()?;

                let head = repo.find_reference("FETCH_HEAD")?;
                let head = head.peel_to_commit()?;

                // reset to the most recent commit
                repo.reset(head.as_object(), ResetType::Hard, None)?;

                Ok::<_, anyhow::Error>(())
            })?;

            repo
        }
        Err(err) => {
            log::info!(
                "Clone failed - code: {:?}, class: {:?}",
                err.code(),
                err.class()
            );
            return Err(err.into());
        }
    };

    log::debug!("Repository cloned or updated");

    // discover files between "then" and now

    let changes = match &continuation {
        Some(commit) => {
            log::info!("Continuing from: {commit}");

            let files = info_span!("continue from", commit).in_scope(|| {
                let start = repo.find_commit(repo.revparse_single(commit)?.id())?;
                let end = repo.head()?.peel_to_commit()?;

                let start = start.tree()?;
                let end = end.tree()?;

                let diff = repo.diff_tree_to_tree(Some(&start), Some(&end), None)?;

                let mut files = HashSet::with_capacity(diff.deltas().len());

                for delta in diff.deltas() {
                    if let Some(path) = delta.new_file().path() {
                        let path = path.to_path_buf();
                        log::debug!("Record {} as changed file", path.display());
                        files.insert(path);
                    }
                }

                Ok::<_, Error>(files)
            })?;

            log::info!("Detected {} changed files", files.len());

            Some(files)
        }
        _ => {
            log::debug!("Ingesting all files");
            None
        }
    };

    /*
    // discover and process files

    let mut path = Cow::Borrowed(path);
    if let Some(base) = path {
        let new_path = path.join(base);

        log::debug!("  Base: {}", path.display());
        log::debug!("Target: {}", new_path.display());

        // ensure that self.path was a relative sub-directory of the repository
        let _ = new_path
            .strip_prefix(path)
            .map_err(|_| Error::Path(base.into()))?;

        path = new_path.into();
    }*/

    //     self.walk(&path, &changes)?;

    let head = repo.head()?;
    let commit = head.peel_to_commit()?.id();
    log::info!("Most recent commit: {commit}");

    // return result

    log::info!("Continuation: {}", commit.to_string());

    Ok(())
}
