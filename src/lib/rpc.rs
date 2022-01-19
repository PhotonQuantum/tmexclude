#![allow(clippy::future_not_send)]
//! RPC facilities.

use std::time::Duration;

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
pub struct Response {
    /// Time elapsed processing this request.
    pub elapsed: Duration,
    /// The main response body.
    pub body: Body,
}

/// Represents an RPC response payload.
#[derive(Debug, Serialize, Deserialize)]
pub enum Body {
    /// A empty success response.
    Empty,
    /// An error response containing a user-friendly error structure.
    Error(SerializedError),
}

impl Body {
    /// Check whether this response indicates a success.
    #[must_use]
    pub const fn success(&self) -> bool {
        matches!(self, Body::Empty)
    }
}

impl<E: Into<Report>> From<E> for Body {
    fn from(e: E) -> Self {
        Self::Error(SerializedError::from_error(e))
    }
}

pub mod server {
    //! RPC server.

    use std::io;
    use std::ops::ControlFlow;
    use std::path::Path;
    use std::time::Instant;

    use actix::Addr;
    use actix_rt::net::UnixListener;
    use actix_rt::System;
    use futures_util::{SinkExt, StreamExt};
    use log::{info, warn};
    use signal_hook::consts::TERM_SIGNALS;
    use signal_hook_tokio::Signals;
    use tokio::sync::mpsc::unbounded_channel;
    use tokio_serde::formats::Bincode;
    use tokio_serde::Framed as SerdeFramed;
    use tokio_util::codec::{Framed, LengthDelimitedCodec};

    use crate::daemon::{Daemon, Pause, ProviderFactory, Reload, Restart};
    use crate::rpc::{Body, Request, Response};

    /// Start the RPC server.
    ///
    /// # Errors
    /// `io::Error` if can't bind to given Unix domain socket.
    pub async fn start_server<F>(uds: impl AsRef<Path>, daemon: Addr<Daemon<F>>) -> io::Result<()>
    where
        F: ProviderFactory,
    {
        let listener = UnixListener::bind(uds)?;
        info!("Server started.");
        let (stop_tx, mut stop_rx) = unbounded_channel();
        let mut signals = Signals::new(TERM_SIGNALS)?;
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
                            let begin = Instant::now();
                            let body = handle_request(&request, &daemon).await;
                            match body {
                                ControlFlow::Continue(body) => {
                                    let resp = Response {
                                        elapsed: begin.elapsed(),
                                        body
                                    };
                                    if let Err(e) = framed.send(resp).await {
                                        warn!("Error when responding to rpc: {}", e);
                                    }
                                }
                                ControlFlow::Break(body) => {
                                    let resp = Response {
                                        elapsed: begin.elapsed(),
                                        body
                                    };
                                    if let Err(e) = framed.send(resp).await {
                                        warn!("Error when responding to rpc: {}", e);
                                    }
                                    let _ = stop_tx.send(());
                                }
                            }
                        }
                    });
                },
                _ = stop_rx.recv() => break,
                _ = signals.next() => break,
                else => ()
            }
        }
        info!("Stopping server.");
        Ok(())
    }

    async fn handle_request<F>(
        request: &Request,
        daemon: &Addr<Daemon<F>>,
    ) -> ControlFlow<Body, Body>
    where
        F: ProviderFactory,
    {
        ControlFlow::Continue(match request {
            Request::Pause => match daemon.send(Pause).await {
                Ok(_) => Body::Empty,
                Err(e) => e.into(),
            },
            Request::Reload => match daemon.send(Reload).await {
                Ok(Ok(_)) => Body::Empty,
                Ok(Err(e)) => e.into(),
                Err(e) => e.into(),
            },
            Request::Restart => match daemon.send(Restart).await {
                Ok(_) => Body::Empty,
                Err(e) => e.into(),
            },
            Request::Shutdown => {
                System::current().stop();
                return ControlFlow::Break(Body::Empty);
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
