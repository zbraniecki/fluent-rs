use async_trait::async_trait;

pub trait SyncFileFetcher {
    fn fetch_file_sync(&self, path: &str) -> std::io::Result<String>;
}

#[async_trait(?Send)]
pub trait AsyncFileFetcher {
    async fn fetch_file_async(&self, path: &str) -> std::io::Result<String>;
}
