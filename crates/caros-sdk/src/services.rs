use crate::{Error, ManagementClient};
use caros_protocol::{Reply, Request, ServiceStatus};

/// Access to administrator-configured providers. Acquisition is restricted by
/// Core to the registered app process and its configured allow-list.
#[derive(Clone, Copy)]
pub struct Services<'a> {
    client: &'a ManagementClient,
}
impl<'a> Services<'a> {
    pub(crate) fn new(client: &'a ManagementClient) -> Self {
        Self { client }
    }
    pub async fn acquire(self, name: impl Into<String>) -> Result<ServiceStatus, Error> {
        self.status_request(Request::AcquireService { name: name.into() })
            .await
    }
    pub async fn status(self, name: impl Into<String>) -> Result<ServiceStatus, Error> {
        self.status_request(Request::ServiceStatus { name: name.into() })
            .await
    }
    pub async fn release(self, name: impl Into<String>) -> Result<(), Error> {
        match self
            .client
            .request(Request::ReleaseService { name: name.into() })
            .await?
        {
            Reply::Accepted => Ok(()),
            _ => Err(Error::Protocol("unexpected service reply".into())),
        }
    }
    async fn status_request(self, request: Request) -> Result<ServiceStatus, Error> {
        match self.client.request(request).await? {
            Reply::Service(status) => Ok(status),
            _ => Err(Error::Protocol("unexpected service reply".into())),
        }
    }
}
