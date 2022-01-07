//! RPC facilities.
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Request {
    command: Command,
}

#[derive(Debug, Serialize, Deserialize)]
enum Command {
    Pause,
    Reload,
    Restart,
    Shutdown,
}

#[derive(Debug, Serialize, Deserialize)]
struct Response {
    success: bool,
    msg: Option<String>,
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
    use log::{debug, info, warn};
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
                    debug!("Connection accepted");
                    let mut framed = SerdeFramed::new(
                        Framed::new(stream, LengthDelimitedCodec::new()),
                        Bincode::<Request, Response>::default(),
                    );
                    let daemon = daemon.clone();
                    let stop_tx = stop_tx.clone();
                    tokio::spawn(async move {
                        while let Some(Ok(request)) = framed.next().await {
                            debug!("Received request: {:?}", request);
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
