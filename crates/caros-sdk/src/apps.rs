use crate::{Error, ManagementClient};
use caros_protocol::{AppId, AppStatus, Reply, Request};

/// App IDs identify entries registered locally by Core; clients cannot supply executables.
#[derive(Clone, Copy)]
pub struct Apps<'a> {
    client: &'a ManagementClient,
}
impl<'a> Apps<'a> {
    pub(crate) fn new(client: &'a ManagementClient) -> Self {
        Self { client }
    }
    pub async fn list(self) -> Result<Vec<AppStatus>, Error> {
        match self.client.request(Request::ListApps).await? {
            Reply::Apps(apps) => Ok(apps),
            _ => Err(unexpected()),
        }
    }
    pub async fn status(self, id: AppId) -> Result<AppStatus, Error> {
        match self.client.request(Request::AppStatus { id }).await? {
            Reply::Status(status) => Ok(status),
            _ => Err(unexpected()),
        }
    }
    /// Accepted means spawn succeeded, not that the app is ready for use.
    pub async fn start(self, id: AppId) -> Result<(), Error> {
        self.command(Request::StartApp { id }).await
    }
    /// Accepted means shutdown was requested. Observe events/status for completion.
    pub async fn stop(self, id: AppId) -> Result<(), Error> {
        self.command(Request::StopApp { id }).await
    }
    async fn command(self, request: Request) -> Result<(), Error> {
        match self.client.request(request).await? {
            Reply::Accepted => Ok(()),
            _ => Err(unexpected()),
        }
    }
}
fn unexpected() -> Error {
    Error::Protocol("unexpected response payload".into())
}
