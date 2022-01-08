use std::path::PathBuf;

use actix_rt::System;
use eyre::Result;

use tmexclude_lib::rpc::client::Client;
use tmexclude_lib::rpc::Request;

use crate::common::ensure_uds_path;

pub fn client(req: Request, uds: Option<PathBuf>) -> Result<()> {
    System::new().block_on(async move {
        let mut client = Client::connect(ensure_uds_path(uds, false)?).await?;
        let resp = client.send(req).await?;
        println!("{:#?}", resp);
        Ok(())
    })
}
