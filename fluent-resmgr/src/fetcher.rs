use async_trait::async_trait;
use fluent_io::FileFetcher;
use std::io;

#[derive(Clone)]
pub struct FSFileFetcher;

#[async_trait]
impl FileFetcher for FSFileFetcher {
    fn fetch_sync(&self, path: &str) -> io::Result<String> {
        std::fs::read_to_string(path)
    }

    async fn fetch(&self, path: &str) -> io::Result<String> {
        let s = tokio::fs::read_to_string(path).await?;
        Ok(s)
    }
}
