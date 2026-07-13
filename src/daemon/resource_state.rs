use anawt::TorrentClient;
use cfg_if::cfg_if;
#[cfg(feature = "ui")]
use freya::prelude::State;
use futures::FutureExt;
use std::{
    convert::Infallible,
    pin::Pin,
    task::{Context, Poll},
};

use crate::{
    config::{AkarekoConfig, I2PRouterConfig},
    db::Repositories,
    server::client::pool::ClientPool,
};

pub trait Resource: Clone + Send + 'static {}
impl<T: Clone + Send + 'static> Resource for T {}
// ==============================================================================
//                                ResourceState
// ==============================================================================
pub enum ResourceState<T, E> {
    Pending,
    Error(E),
    Loading,
    Loaded(T),
}

impl<T, E> Default for ResourceState<T, E> {
    fn default() -> Self {
        Self::Pending
    }
}

impl<T: Clone, E: Clone> Clone for ResourceState<T, E> {
    fn clone(&self) -> Self {
        match self {
            ResourceState::Pending => ResourceState::Pending,
            ResourceState::Error(e) => ResourceState::Error(e.clone()),
            ResourceState::Loading => ResourceState::Loading,
            ResourceState::Loaded(l) => ResourceState::Loaded(l.clone()),
        }
    }
}

impl<T, E> ResourceState<T, E> {
    pub fn unwrap_ref(&self) -> &T {
        match self {
            ResourceState::Pending => panic!("ResourceState::Pending"),
            ResourceState::Error(_) => panic!("ResourceState::Error"),
            ResourceState::Loading => panic!("ResourceState::Loading"),
            ResourceState::Loaded(t) => t,
        }
    }

    pub fn mut_unwrap_ref(&mut self) -> &mut T {
        match self {
            ResourceState::Pending => panic!("ResourceState::Pending"),
            ResourceState::Error(_) => panic!("ResourceState::Error"),
            ResourceState::Loading => panic!("ResourceState::Loading"),
            ResourceState::Loaded(t) => t,
        }
    }
}

// ==============================================================================
//                             Resources Manager
// ==============================================================================

#[derive(Default)]
pub struct AppResourcesManager {
    pub(super) config: ResourceStateManager<AkarekoConfig, Infallible>,
    pub(super) i2p_router_config: ResourceStateManager<I2PRouterConfig, Infallible>,
    pub(super) client: ResourceStateManager<ClientPool, ()>,
    pub(super) server: ResourceStateManager<(), ()>,
    pub(super) repositories: ResourceStateManager<Repositories, Infallible>,
    pub(super) torrent_client: ResourceStateManager<TorrentClient, ()>,
}

impl AppResourcesManager {
    #[cfg(feature = "ui")]
    pub fn get_resources(&self) -> AppResources {
        AppResources {
            config: self.config.state(),
            i2p_router_config: self.i2p_router_config.state(),
            client: self.client.state(),
            server: self.server.state(),
            repositories: self.repositories.state(),
            torrent_client: self.torrent_client.state(),
        }
    }
}

pub struct ResourceStateManager<T: Send + 'static, E: Send + 'static> {
    #[cfg(feature = "ui")]
    state: State<ResourceState<T, E>>,
    #[cfg(not(feature = "ui"))]
    state: ResourceState<T, E>,
    load_hdl: Option<tokio::task::JoinHandle<Result<T, E>>>,
}

impl<T: Resource, E: Resource> ResourceStateManager<T, E> {
    #[cfg(feature = "ui")]
    pub fn state(&self) -> State<ResourceState<T, E>> {
        self.state
    }

    pub fn get_value(&self) -> Option<T> {
        cfg_if! {
            if #[cfg(feature = "ui")] {
                match &*self.state.peek() {
                    ResourceState::Loaded(v) => Some(v.clone()),
                    _ => None,
                }
            }
            else {
                match &self.state {
                    ResourceState::Loaded(v) => Some(v.clone()),
                    _ => None,
                }
            }
        }
    }

    fn abort_load(&mut self) {
        cfg_if! {
            if #[cfg(feature = "ui")] {
                let state = &*self.state.peek();
            }
            else {
                let state = &self.state;
            }
        };
        if let ResourceState::Loading = state {
            if let Some(hdl) = &self.load_hdl {
                hdl.abort();
                self.load_hdl = None;
            };
        }
    }

    pub fn load<F>(&mut self, future: F)
    where
        F: Future<Output = Result<T, E>> + Send + 'static,
    {
        self.abort_load();
        let join_handle = tokio::spawn(future);
        cfg_if! {
            if #[cfg(feature = "ui")] {
                *self.state.write() = ResourceState::Loading;
            }
            else {
                self.state = ResourceState::Loading;
            }
        };
        self.load_hdl = Some(join_handle);
    }

    pub fn force_load(&mut self, value: T) {
        self.abort_load();

        cfg_if! {
            if #[cfg(feature = "ui")] {
                *self.state.write() =  ResourceState::Loaded(value)
            }
            else {
                self.state = ResourceState::Loaded(value)
            }
        };
    }

    pub fn unload(&mut self) {
        self.abort_load();
        cfg_if! {
            if #[cfg(feature = "ui")] {
                *self.state.write() = ResourceState::Pending;
            }
            else {
                self.state = ResourceState::Pending;
            }
        };
    }
}

impl<T: Resource, E: Resource> Default for ResourceStateManager<T, E> {
    fn default() -> Self {
        Self {
            #[cfg(feature = "ui")]
            state: State::create_global(Default::default()),
            #[cfg(not(feature = "ui"))]
            state: Default::default(),
            load_hdl: Default::default(),
        }
    }
}

impl<T: Resource, E: Resource> Clone for ResourceStateManager<T, E> {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            load_hdl: None,
        }
    }
}

impl<T: Resource, E: Resource> Future for &mut ResourceStateManager<T, E> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        cfg_if! {
            if #[cfg(feature = "ui")] {
                let state = this.state.peek().clone();
            } else {
                let state = this.state.clone();
            }
        };

        match (state, &mut this.load_hdl) {
            (ResourceState::Loading, Some(hdl)) => match hdl.poll_unpin(cx) {
                Poll::Ready(v) => {
                    match v {
                        Ok(Ok(v)) => {
                            this.load_hdl = None;
                            cfg_if! {
                                if #[cfg(feature = "ui")] {
                                    *this.state.write() = ResourceState::Loaded(v);
                                } else {
                                    this.state = ResourceState::Loaded(v);
                                }
                            }
                        }
                        Ok(Err(e)) => {
                            this.load_hdl = None;
                            cfg_if! {
                                if #[cfg(feature = "ui")] {
                                    *this.state.write() = ResourceState::Error(e);
                                } else {
                                    this.state = ResourceState::Error(e);
                                }
                            }
                        }
                        Err(_) => todo!(),
                    }
                    Poll::Ready(())
                }
                Poll::Pending => Poll::Pending,
            },
            _ => Poll::Pending,
        }
    }
}

// ==============================================================================
//                               App Resources (UI)
// ==============================================================================

#[cfg(feature = "ui")]
pub use ui::AppResources;
#[cfg(feature = "ui")]
mod ui {
    use std::convert::Infallible;

    use anawt::TorrentClient;
    use freya::prelude::*;

    use crate::{
        config::{AkarekoConfig, I2PRouterConfig},
        db::Repositories,
        server::client::pool::ClientPool,
    };

    use super::ResourceState;

    #[derive(Clone, Copy)]
    pub struct AppResources {
        pub config: State<ResourceState<AkarekoConfig, Infallible>>,
        pub i2p_router_config: State<ResourceState<I2PRouterConfig, Infallible>>,
        pub client: State<ResourceState<ClientPool, ()>>,
        pub server: State<ResourceState<(), ()>>,
        pub repositories: State<ResourceState<Repositories, Infallible>>,
        pub torrent_client: State<ResourceState<TorrentClient, ()>>,
    }

    impl AppResources {
        pub fn register_context(&self) {
            use_provide_context(|| self.client);
            use_provide_context(|| self.config);
            use_provide_context(|| self.i2p_router_config);
            use_provide_context(|| self.repositories);
            // use_provide_context(|| self.server);
            use_provide_context(|| self.torrent_client);
        }

        pub fn get_client() -> ResourceState<ClientPool, ()> {
            let state: State<ResourceState<ClientPool, ()>> = consume_context();
            state.read().clone()
        }
        pub fn get_config() -> ResourceState<AkarekoConfig, Infallible> {
            let state: State<ResourceState<AkarekoConfig, Infallible>> = consume_context();
            state.read().clone()
        }
        pub fn get_i2p_router_config() -> ResourceState<I2PRouterConfig, Infallible> {
            let state: State<ResourceState<I2PRouterConfig, Infallible>> = consume_context();
            state.read().clone()
        }
        // pub fn get_server() -> State<ResourceState<(), ()>> {
        //     consume_context()
        // }
        pub fn get_repositories() -> ResourceState<Repositories, Infallible> {
            let state: State<ResourceState<Repositories, Infallible>> = consume_context();
            state.read().clone()
        }
        pub fn get_torrent_client() -> ResourceState<TorrentClient, ()> {
            let state: State<ResourceState<TorrentClient, ()>> = consume_context();
            state.read().clone()
        }
    }
}
