#![allow(clippy::future_not_send)]
//! RPC facilities.

use eyre::Report;
use serde::{Deserialize, Serialize};

use crate::errors::SerializedError;

/// Represents an RPC request.
#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    /// Pause daemon.
    Pause,
    /// Reload config and restart daemon.
    Reload,
    /// Restart daemon. This method doesn't reload config.
    Restart,
    /// Terminate daemon.
    Shutdown,
}

/// Represents an RPC response.
#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    /// A empty success response.
    Empty,
    /// An error response containing a user-friendly error structure.
    Error(SerializedError),
}

impl Response {
    /// Check whether this response indicates a success.
    #[must_use]
    pub const fn success(&self) -> bool {
        matches!(self, Response::Empty)
    }
}

impl<E: Into<Report>> From<E> for Response {
    fn from(e: E) -> Self {
        Self::Error(SerializedError::from_error(e))
    }
}

pub mod server {
    //! RPC server.

    use std::io;
    use std::ops::ControlFlow;
    use std::path::PathBuf;

    use actix::Addr;
    use actix_rt::net::UnixListener;
    use actix_rt::System;
    use futures_util::{SinkExt, StreamExt};
    use log::{info, warn};
    use tokio::sync::mpsc::unbounded_channel;
    use tokio_serde::formats::Bincode;
    use tokio_serde::Framed as SerdeFramed;
    use tokio_util::codec::{Framed, LengthDelimitedCodec};

    use crate::daemon::{Daemon, Pause, ProviderFactory, Reload, Restart};
    use crate::rpc::{Request, Response};

    /// Start the RPC server.
    ///
    /// # Errors
    /// `io::Error` if can't bind to given Unix domain socket.
    pub async fn start_server<F>(uds: PathBuf, daemon: Addr<Daemon<F>>) -> io::Result<()>
    where
        F: ProviderFactory,
    {
        let listener = UnixListener::bind(uds)?;
        info!("Server started.");
        let (stop_tx, mut stop_rx) = unbounded_channel();
        loop {
            tokio::select! {
                Ok((stream, _)) = listener.accept() => {
                    info!("Connection accepted");
                    let mut framed = SerdeFramed::new(
                        Framed::new(stream, LengthDelimitedCodec::new()),
                        Bincode::<Request, Response>::default(),
                    );
                    let daemon = daemon.clone();
                    let stop_tx = stop_tx.clone();
                    tokio::spawn(async move {
                        while let Some(Ok(request)) = framed.next().await {
                            info!("Received request: {:?}", request);
                            let resp = handle_request(&request, &daemon).await;
                            match resp {
                                ControlFlow::Continue(r) => {
                                    if let Err(e) = framed.send(r).await {
                                        warn!("Error when responding to rpc: {}", e);
                                    }
                                }
                                ControlFlow::Break(r) => {
                                    if let Err(e) = framed.send(r).await {
                                        warn!("Error when responding to rpc: {}", e);
                                    }
                                    let _ = stop_tx.send(());
                                }
                            }
                        }
                    });
                },
                _ = stop_rx.recv() => break,
                else => ()
            }
        }
        Ok(())
    }

    async fn handle_request<F>(
        request: &Request,
        daemon: &Addr<Daemon<F>>,
    ) -> ControlFlow<Response, Response>
    where
        F: ProviderFactory,
    {
        ControlFlow::Continue(match request {
            Request::Pause => match daemon.send(Pause).await {
                Ok(_) => Response::Empty,
                Err(e) => e.into(),
            },
            Request::Reload => match daemon.send(Reload).await {
                Ok(Ok(_)) => Response::Empty,
                Ok(Err(e)) => e.into(),
                Err(e) => e.into(),
            },
            Request::Restart => match daemon.send(Restart).await {
                Ok(_) => Response::Empty,
                Err(e) => e.into(),
            },
            Request::Shutdown => {
                System::current().stop();
                return ControlFlow::Break(Response::Empty);
            }
        })
    }
}

pub mod client {
    //! RPC client.
    use std::io;
    use std::io::ErrorKind;
    use std::path::Path;

    use futures_util::{SinkExt, StreamExt};
    use tokio::net::UnixStream;
    use tokio_serde::formats::Bincode;
    use tokio_serde::Framed as SerdeFramed;
    use tokio_util::codec::{Framed, LengthDelimitedCodec};

    use crate::rpc::{Request, Response};

    type RawFrame = Framed<UnixStream, LengthDelimitedCodec>;
    type Codec = Bincode<Response, Request>;
    type Frame = SerdeFramed<RawFrame, Response, Request, Codec>;

    /// RPC client.
    pub struct Client {
        stream: Frame,
    }

    impl Client {
        /// Connect to given uds.
        ///
        /// # Errors
        /// `io::Error` if fails to connect to given uds.
        pub async fn connect(uds: impl AsRef<Path>) -> io::Result<Self> {
            let raw = UnixStream::connect(uds).await?;
            Ok(Self {
                stream: SerdeFramed::new(
                    Framed::new(raw, LengthDelimitedCodec::new()),
                    Bincode::<Response, Request>::default(),
                ),
            })
        }
        /// Send a message and get its result.
        ///
        /// # Errors
        /// `io::Error` if unable to send
        pub async fn send(&mut self, msg: Request) -> io::Result<Response> {
            self.stream.send(msg).await?;
            self.stream
                .next()
                .await
                .ok_or_else(|| io::Error::from(ErrorKind::ConnectionAborted))?
        }
    }
}
