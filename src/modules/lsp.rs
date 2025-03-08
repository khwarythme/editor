use lsp_client::{transport::io_transport, LspClient};
use lsp_types::{notification::*, request::*, *};
use tokio::{
    process::{Child, Command},
    sync::oneshot,
};

use anyhow::{Context, Result};
use std::process::Stdio;
use std::str::FromStr;

use super::file::FileBuffer;

pub struct client {
    client: LspClient,
    process: Child,
}

impl client {
    pub fn new(client_type: String) -> Self {
        let mut process = Command::new(client_type)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let stdin = process.stdin.take().unwrap();
        let stdout = process.stdout.take().unwrap();
        let (tx, rx) = io_transport(stdin, stdout);
        let client = LspClient::new(tx, rx);
        Self { client, process }
    }
    #[tokio::main]
    pub async fn run(&mut self, buf: &FileBuffer) -> Result<()> {
        let mut subscriber = self.client.subscribe_to_method::<Progress>().await?;
        let (indexed_tx, indexed_rx) = oneshot::channel();

        tokio::spawn(async move {
            while let Some(msg) = subscriber.next().await {
                let params = msg.unwrap();
                if matches!(params.token, NumberOrString::String(s) if s == "rustAnalyzer/Indexing")
                    && matches!(
                        params.value,
                        ProgressParamsValue::WorkDone(WorkDoneProgress::End(_))
                    )
                {
                    indexed_tx.send(()).unwrap();
                    break;
                }
            }
            subscriber.unsubscribe().await.unwrap();
        });

        let sourcefile = buf.get_path();
        let source_uri = Uri::from_str(&format!("file://{}", sourcefile))?;
        let initialize_params = InitializeParams {
            capabilities: ClientCapabilities {
                workspace: Some(WorkspaceClientCapabilities {
                    workspace_folders: Some(true),
                    ..Default::default()
                }),
                window: Some(WindowClientCapabilities {
                    work_done_progress: Some(true),
                    ..Default::default()
                }),
                ..Default::default()
            },
            workspace_folders: Some(vec![WorkspaceFolder {
                name: "root".to_string(),
                uri: source_uri.clone(),
            }]),
            ..Default::default()
        };
        self.client.initialize(initialize_params).await?;
        self.client.initialized().await?;

        Ok(())
    }
}
