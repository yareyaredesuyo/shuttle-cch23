use super::error::AppError;
use axum::{routing::post, Router};
use bytes::{Buf as _, Bytes};
use git2::Repository;
use tar::Archive;
// use tracing::info;

pub fn task() -> Router {
    Router::new()
        .route("/archive_files", post(archive_files_route))
        .route("/archive_files_size", post(archive_files_size_route))
        .route("/cookie", post(git_cookie_route))
}

async fn archive_files_route(body: Bytes) -> Result<String, AppError> {
    let mut archive = Archive::new(body.reader());
    let count = archive.entries().unwrap().count();
    Ok(count.to_string())
}

async fn archive_files_size_route(body: Bytes) -> Result<String, AppError> {
    let mut archive = Archive::new(body.reader());
    let total_size = archive
        .entries()
        .unwrap()
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.size())
        .sum::<u64>();

    Ok(total_size.to_string())
}

async fn git_cookie_route(body: Bytes) -> anyhow::Result<String, AppError> {
    let temp_dir = tempfile::tempdir()?;
    Archive::new(body.reader()).unpack(temp_dir.path())?;

    let repo = Repository::open(temp_dir.path()).unwrap();
    let branch = repo.find_branch("christmas", git2::BranchType::Local)?;
    let head_commit = branch.get().peel_to_commit()?;
    // info!("{:?}", head_commit);

    let mut commit = head_commit;
    while let Ok(tree) = commit.tree() {
        // info!("tree {:?}", tree);
        let mut find_cookie = false;

        tree.walk(git2::TreeWalkMode::PreOrder, |_, entry| {
            // info!("{:?}", entry.to_object(&repo));
            if entry.name() == Some("santa.txt")
                && std::str::from_utf8(entry.to_object(&repo).unwrap().as_blob().unwrap().content())
                    .unwrap()
                    .contains("COOKIE")
            {
                find_cookie = true;
                git2::TreeWalkResult::Abort
            } else {
                git2::TreeWalkResult::Ok
            }
        })?;
        if find_cookie {
            return {
                Ok(format!(
                    "{} {}",
                    commit.author().name().unwrap(),
                    commit.id()
                ))
            };
        }
        if let Some(parent) = commit.parents().next() {
            commit = parent;
        } else {
            break;
        }
    }

    Err(anyhow::anyhow!("not found"))?
}
