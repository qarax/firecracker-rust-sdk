use std::{borrow::Cow, path::{Path, PathBuf}};

use crate::{
    client::{HttpClient, TokioIo},
    error::Error,
    models::{
        BootSource, Drive, InstanceInfo, MachineConfiguration, NetworkInterface,
        SnapshotCreateParams, SnapshotLoadParams, Vm,
    },
};
use bytes::{Bytes, BytesMut};
use futures::stream::StreamExt;
use http_body_util::{Empty, Full, combinators::BoxBody};
use hyper::{Request, Response, body::Incoming};
use tokio::{fs, net::UnixStream};
use tracing::{debug, info};
use uuid::Uuid;

/// Configuration for a [`Machine`].
///
/// * `vm_id`      – UUID of the microVM (used to name the tmux session).
/// * `socket_path`– Path to the Firecracker API Unix domain socket.
/// * `exec_path`  – Path to the `firecracker` binary.
#[derive(Clone, Debug)]
pub struct MachineConfig<'m> {
    pub vm_id: Uuid,
    pub socket_path: Cow<'m, Path>,
    pub exec_path: Cow<'m, Path>,
}

/// A handle to a Firecracker VMM process.
///
/// Use [`Machine::start`] to launch a new Firecracker process and obtain a
/// connected handle, or [`Machine::connect`] to attach to an already-running
/// instance.
pub struct Machine<'m> {
    config: MachineConfig<'m>,
    client: HttpClient,
}

/// A configured microVM running inside a [`Machine`].
pub struct MicroVm<'m> {
    machine: Machine<'m>,
}

impl<'m> Machine<'m> {
    /// Launches a new Firecracker VMM process in a tmux session and connects
    /// to its API socket.
    pub async fn start(config: MachineConfig<'_>) -> Result<Machine<'_>, Error> {
        if config.socket_path.exists() {
            fs::remove_file(&config.socket_path).await?;
        }

        let tmux_cmd = Self::generate_firecracker_cmd(&config);

        let output = tokio::process::Command::new("sh")
            .arg("-c")
            .arg(&tmux_cmd)
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(std::io::Error::other(format!(
                "Failed to start Firecracker in tmux: {}",
                stderr
            ))
            .into());
        }

        let client = Self::build_client(&config.socket_path).await?;

        info!(
            "Firecracker started in tmux session 'vm_{}'. Attach with:\n  tmux attach -t vm_{}",
            config.vm_id, config.vm_id
        );

        Ok(Machine { config, client })
    }

    /// Connects to an already-running Firecracker instance via its API socket.
    pub async fn connect(config: MachineConfig<'m>) -> Result<MicroVm<'m>, Error> {
        let client = Self::build_client(&config.socket_path).await?;
        Ok(MicroVm {
            machine: Machine { config, client },
        })
    }

    /// Attaches to a Firecracker process you spawned yourself.
    ///
    /// Waits for the socket to become available (up to 5 s), then returns a
    /// [`Machine`] ready for pre-boot configuration. Unlike [`Machine::start`],
    /// this does **not** spawn a process — process lifecycle is the caller's
    /// responsibility.
    pub async fn attach(socket_path: impl Into<PathBuf>) -> Result<Machine<'static>, Error> {
        let socket_path = socket_path.into();
        let client = Self::build_client(&socket_path).await?;
        Ok(Machine {
            config: MachineConfig {
                vm_id: Uuid::nil(),
                socket_path: Cow::Owned(socket_path),
                exec_path: Cow::Owned(PathBuf::new()),
            },
            client,
        })
    }

    // ── Pre-boot configuration helpers ───────────────────────────────────────

    /// Configures the boot source (kernel + optional initrd + boot args).
    pub async fn put_boot_source(&mut self, boot_source: &BootSource) -> Result<(), Error> {
        self.put("/boot-source", boot_source).await
    }

    /// Adds or replaces a drive.
    pub async fn put_drive(&mut self, drive_id: &str, drive: &Drive) -> Result<(), Error> {
        self.put(&format!("/drives/{drive_id}"), drive).await
    }

    /// Configures the machine (vCPU count, memory, SMT …).
    pub async fn put_machine_config(
        &mut self,
        machine_config: &MachineConfiguration,
    ) -> Result<(), Error> {
        self.put("/machine-config", machine_config).await
    }

    /// Adds a network interface (pre-boot only).
    pub async fn put_network_interface(
        &mut self,
        iface_id: &str,
        iface: &NetworkInterface,
    ) -> Result<(), Error> {
        self.put(&format!("/network-interfaces/{iface_id}"), iface)
            .await
    }

    /// Starts the microVM (equivalent to `InstanceStart` action).
    ///
    /// Consumes the [`Machine`] and returns a [`MicroVm`] handle for
    /// post-boot operations. The connection is refreshed so the returned
    /// [`MicroVm`] has a clean HTTP/1.1 connection.
    pub async fn start_instance(mut self) -> Result<MicroVm<'m>, Error> {
        #[derive(Serialize)]
        struct Action {
            action_type: &'static str,
        }
        self.put("/actions", &Action { action_type: "InstanceStart" })
            .await?;
        self.client = Self::build_client(&self.config.socket_path).await?;
        Ok(MicroVm { machine: self })
    }

    /// Loads a snapshot and resumes the VM in one step.
    ///
    /// Consumes the [`Machine`] (which must be freshly spawned with no prior
    /// configuration) and returns a running [`MicroVm`]. Set
    /// `params.resume_vm = Some(true)` to resume automatically.
    pub async fn load_and_resume(mut self, params: &SnapshotLoadParams) -> Result<MicroVm<'m>, Error> {
        self.put("/snapshot/load", params).await?;
        self.client = Self::build_client(&self.config.socket_path).await?;
        Ok(MicroVm { machine: self })
    }

    /// Wraps this pre-boot [`Machine`] in a [`MicroVm`] without sending
    /// `InstanceStart`. Use for recovery when the VM is already running.
    pub fn assume_running(self) -> MicroVm<'m> {
        MicroVm { machine: self }
    }

    // ── Internal helpers ──────────────────────────────────────────────────────

    fn generate_firecracker_cmd(config: &MachineConfig) -> String {
        let fc_cmd = format!(
            "{} --api-sock {}",
            config
                .exec_path
                .to_str()
                .expect("exec_path is not valid UTF-8"),
            config.socket_path.to_string_lossy()
        );
        let session_name = format!("vm_{}", config.vm_id);
        format!("tmux new-session -d -s '{session_name}' {fc_cmd}")
    }

    async fn build_client(socket_path: &Path) -> Result<HttpClient, Error> {
        info!("connecting HTTP client to Firecracker at {socket_path:?}...");

        let max_retries = 50; // 5 s total (50 × 100 ms)
        let mut retries = 0;
        let stream = loop {
            match UnixStream::connect(socket_path).await {
                Ok(stream) => break stream,
                Err(_e) if retries < max_retries => {
                    retries += 1;
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    if retries % 10 == 0 {
                        info!(
                            "Still waiting for Firecracker socket… ({}/{})",
                            retries, max_retries
                        );
                    }
                }
                Err(e) => return Err(e.into()),
            }
        };

        let io = TokioIo::new(stream);
        let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await?;

        tokio::spawn(conn);
        sender.ready().await?;

        info!("HTTP client connected to Firecracker at {socket_path:?}");
        Ok(HttpClient::new(sender))
    }

    /// Generic PUT helper – serialises `body` as JSON and asserts a 2xx status.
    async fn put<T: serde::Serialize>(&mut self, path: &str, body: &T) -> Result<(), Error> {
        let json = serde_json::to_vec(body)?;
        let request = Request::builder()
            .method("PUT")
            .uri(format!("http://localhost{path}"))
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .body(BoxBody::new(Full::new(Bytes::from(json))))?;

        let response = self.client.sender().send_request(request).await?;
        debug!("{:#?}", response);

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = read_response_body(response).await?;
            return Err(Error::FirecrackerApiError(status, body));
        }
        Ok(())
    }
}

impl MicroVm<'_> {
    /// Returns general info about the running microVM instance.
    pub async fn describe_instance(&mut self) -> Result<InstanceInfo, Error> {
        let request = Request::builder()
            .method("GET")
            .uri("http://localhost/")
            .header("Accept", "application/json")
            .body(BoxBody::new(Empty::new()))?;

        let response = self.machine.client.sender().send_request(request).await?;
        debug!("{:#?}", response);

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = read_response_body(response).await?;
            return Err(Error::FirecrackerApiError(status, body));
        }

        let body = read_response_body(response).await?;
        Ok(serde_json::from_str(&body)?)
    }

    /// Pauses or resumes the microVM.
    pub async fn patch_vm(&mut self, state: &Vm) -> Result<(), Error> {
        self.machine.patch("/vm", state).await
    }

    /// Creates a snapshot (post-boot, VM must be paused).
    pub async fn create_snapshot(&mut self, params: &SnapshotCreateParams) -> Result<(), Error> {
        self.machine.put("/snapshot/create", params).await
    }

    /// Sends Ctrl+Alt+Del to the VM.
    pub async fn send_ctrl_alt_del(&mut self) -> Result<(), Error> {
        #[derive(Serialize)]
        struct Action {
            action_type: &'static str,
        }
        self.machine
            .put(
                "/actions",
                &Action {
                    action_type: "SendCtrlAltDel",
                },
            )
            .await
    }

    /// Loads a snapshot (only valid before configuring other resources).
    pub async fn load_snapshot(&mut self, params: &SnapshotLoadParams) -> Result<(), Error> {
        self.machine.put("/snapshot/load", params).await
    }

    /// Returns the Firecracker version string.
    pub async fn get_version(&mut self) -> Result<String, Error> {
        let request = Request::builder()
            .method("GET")
            .uri("http://localhost/version")
            .header("Accept", "application/json")
            .body(BoxBody::new(Empty::new()))?;

        let response = self.machine.client.sender().send_request(request).await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = read_response_body(response).await?;
            return Err(Error::FirecrackerApiError(status, body));
        }

        let body = read_response_body(response).await?;
        let v: serde_json::Value = serde_json::from_str(&body)?;
        Ok(v["firecracker_version"]
            .as_str()
            .unwrap_or_default()
            .to_owned())
    }
}

impl Machine<'_> {
    /// Generic PATCH helper.
    async fn patch<T: serde::Serialize>(&mut self, path: &str, body: &T) -> Result<(), Error> {
        let json = serde_json::to_vec(body)?;
        let request = Request::builder()
            .method("PATCH")
            .uri(format!("http://localhost{path}"))
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .body(BoxBody::new(Full::new(Bytes::from(json))))?;

        let response = self.client.sender().send_request(request).await?;
        debug!("{:#?}", response);

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = read_response_body(response).await?;
            return Err(Error::FirecrackerApiError(status, body));
        }
        Ok(())
    }
}

async fn read_response_body(response: Response<Incoming>) -> Result<String, Error> {
    let mut body_bytes = http_body_util::BodyStream::new(response.into_body());
    let mut bytes = BytesMut::new();
    while let Some(chunk) = body_bytes.next().await {
        let chunk = chunk?;
        if let Ok(data) = chunk.into_data() {
            bytes.extend_from_slice(&data);
        }
    }
    String::from_utf8(bytes.to_vec()).map_err(|e| Error::Other(e.to_string()))
}
