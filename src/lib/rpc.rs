//! RPC facilities.
use serde::{Deserialize, Serialize};

/// Represents an RPC request.
#[derive(Debug, Serialize, Deserialize)]
pub struct Request {
    /// RPC command.
    pub command: Command,
}

/// An RPC command.
#[derive(Debug, Serialize, Deserialize)]
pub enum Command {
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
    /// If the call is success or not.
    pub success: bool,
    /// Supplemental messages.
    pub msg: Option<String>,
}

pub mod server {
    //! RPC server.
    use std::error::Error;
    use std::io;
    use std::ops::ControlFlow;
    use std::path::PathBuf;

    use actix::Addr;
    use actix_rt::net::UnixListener;
    use actix_rt::System;
    use figment::Provider;
    use futures_util::{SinkExt, StreamExt};
    use log::{info, warn};
    use tokio::sync::mpsc::unbounded_channel;
    use tokio_serde::formats::Bincode;
    use tokio_serde::Framed as SerdeFramed;
    use tokio_util::codec::{Framed, LengthDelimitedCodec};

    use crate::daemon::{Daemon, Pause, Reload, Restart};
    use crate::rpc::{Command, Request, Response};
    use crate::utils::TypeEq;

    /// Start the RPC server.
    ///
    /// # Errors
    /// `io::Error` if can't bind to given Unix domain socket.
    pub async fn start_server<F, O, E, P>(uds: PathBuf, daemon: Addr<Daemon<F>>) -> io::Result<()>
    where
        F: Fn() -> O + Unpin + 'static,
        O: TypeEq<Rhs = Result<P, E>>,
        E: 'static + Error + Send + Sync,
        P: Provider,
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

    async fn handle_request<F, O, E, P>(
        request: &Request,
        daemon: &Addr<Daemon<F>>,
    ) -> ControlFlow<Response, Response>
    where
        F: Fn() -> O + Unpin + 'static,
        O: TypeEq<Rhs = Result<P, E>>,
        E: 'static + Error + Send + Sync,
        P: Provider,
    {
        ControlFlow::Continue(match request.command {
            Command::Pause => match daemon.send(Pause).await {
                Ok(_) => Response {
                    success: true,
                    msg: None,
                },
                Err(e) => Response {
                    success: false,
                    msg: Some(e.to_string()),
                },
            },
            Command::Reload => match daemon.send(Reload).await {
                Ok(Ok(_)) => Response {
                    success: true,
                    msg: None,
                },
                Ok(Err(e)) => Response {
                    success: false,
                    msg: Some(e.to_string()),
                },
                Err(e) => Response {
                    success: false,
                    msg: Some(e.to_string()),
                },
            },
            Command::Restart => match daemon.send(Restart).await {
                Ok(_) => Response {
                    success: true,
                    msg: None,
                },
                Err(e) => Response {
                    success: false,
                    msg: Some(e.to_string()),
                },
            },
            Command::Shutdown => {
                System::current().stop();
                return ControlFlow::Break(Response {
                    success: true,
                    msg: None,
                });
            }
        })
    }
}

pub mod client {
    //! RPC client.
    use std::io;
    use std::io::ErrorKind;
    use std::path::PathBuf;

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
        pub async fn connect(uds: PathBuf) -> io::Result<Self> {
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
