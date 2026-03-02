use tokio::sync::Semaphore;

use crate::server::client::AkarekoClient;

#[derive(Clone)]
pub struct ClientPool {
    client: AkarekoClient,
    permits: std::sync::Arc<Semaphore>,
}

impl ClientPool {
    pub fn new(client: AkarekoClient, size: u16) -> Self {
        Self {
            client,
            permits: std::sync::Arc::new(Semaphore::new(size as usize)),
        }
    }

    pub async fn get_client(self) -> PooledClient {
        PooledClient {
            client: self.client,
            _permit: self.permits.acquire_owned().await.unwrap(),
        }
    }
}

pub struct PooledClient {
    client: AkarekoClient,
    _permit: tokio::sync::OwnedSemaphorePermit,
}

impl std::ops::Deref for PooledClient {
    type Target = AkarekoClient;
    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

impl std::ops::DerefMut for PooledClient {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.client
    }
}
