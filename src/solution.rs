use crate::statement::*;
use async_trait::async_trait;

use futures::stream::{AbortHandle, Abortable};
use tokio::task::JoinHandle;

type DownloadHandle = JoinHandle<Result<Result<Binary, ServerError>, futures::stream::Aborted>>;
type Download = (ServerName, DownloadHandle, AbortHandle);

#[async_trait]
pub trait Solution {
    async fn solve(repositories: Vec<ServerName>) -> Option<Binary>;
}

pub struct Solution0;

impl Solution0 {
    fn spawn_download(server_name: ServerName) -> Download {
        let (abort_handle, abort_registration) = AbortHandle::new_pair();

        let server = server_name.clone();
        let join_handle: DownloadHandle = tokio::spawn(Abortable::new(
            async move {
                let name = server.0.clone();
                println!("-> started -> {}", name);
                let result = download(server).await;
                println!("-> result -> {} : {:?}", name, result);
                result
            },
            abort_registration,
        ));
        (server_name, join_handle, abort_handle)
    }
}

#[async_trait]
impl Solution for Solution0 {
    async fn solve(repositories: Vec<ServerName>) -> Option<Binary> {
        let mut handles: Vec<Download> = Vec::new();

        for server in repositories {
            handles.push(Self::spawn_download(server));
        }

        let mut result = None;
        loop {
            // No tasks or successful download
            if handles.is_empty() || result.is_some() {
                break;
            }

            let mut finished: Vec<usize> = Vec::new();
            for idx in 0..handles.len() {
                let (_, handle, _) = &handles[idx];
                if handle.is_finished() {
                    finished.push(idx);
                }
            }

            for idx in finished {
                let (server_name, handle, _) = handles.remove(idx);
                match handle.await {
                    Ok(Ok(Ok(binary))) => {
                        println!("# \\(^_^)/ successful for {}", server_name.0);
                        result = Some(binary);
                        break;
                    }
                    Ok(Ok(Err(err))) => {
                        println!("# RETRY failed for {} {err:?}", server_name.0);
                        handles.push(Self::spawn_download(server_name));
                    }
                    Ok(Err(_)) => {
                        println!("# aborted for {}", server_name.0);
                    }
                    Err(err) => {
                        println!("Join error for {} {err:?}", server_name.0);
                    }
                }
            }
        }

        // Found some download thus we should cancel all others
        if result.is_some() {
            for (name, _, abort_handle) in handles {
                abort_handle.abort();
                println!("@ aborting {}", name.0);
            }
        }

        result
    }
}
