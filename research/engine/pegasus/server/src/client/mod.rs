use std::cell::RefCell;
use std::fmt::{Debug, Display, Formatter};
use std::pin::Pin;
use std::task::{Context, Poll};

use futures::stream::{BoxStream, SelectAll};
use futures::{Stream, StreamExt};
use pegasus::{JobConf, ServerConf};

use crate::pb::job_config::Servers;
use crate::pb::job_service_client::JobServiceClient;
use crate::pb::{BinaryResource, Empty, JobConfig, JobRequest, ServerList};
use crate::service::JobDesc;

pub enum JobError {
    InvalidConfig(String),
    RPCError(tonic::Status),
}

impl Debug for JobError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            JobError::InvalidConfig(msg) => write!(f, "Invalid config: {}", msg),
            JobError::RPCError(status) => write!(f, "RPCError: {}", status),
        }
    }
}

impl Display for JobError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}

pub struct RPCJobClient {
    conns: Vec<Option<RefCell<JobServiceClient<tonic::transport::Channel>>>>,
}

impl RPCJobClient {
    pub fn new() -> Self {
        RPCJobClient { conns: vec![] }
    }

    pub async fn connect<D>(&mut self, server_id: u64, url: D) -> Result<(), tonic::transport::Error>
    where
        D: std::convert::TryInto<tonic::transport::Endpoint>,
        D::Error: Into<tonic::codegen::StdError>,
    {
        let client = JobServiceClient::connect(url).await?;
        while server_id as usize >= self.conns.len() {
            self.conns.push(None);
        }
        self.conns[server_id as usize] = Some(RefCell::new(client));
        Ok(())
    }

    pub async fn submit(
        &mut self, config: JobConf, job: JobDesc,
    ) -> Result<BoxStream<'static, Result<Option<Vec<u8>>, tonic::Status>>, JobError> {
        let mut remotes = vec![];
        let servers = match config.servers() {
            ServerConf::Local => {
                for conn in self.conns.iter() {
                    if let Some(c) = conn {
                        remotes.push(c);
                        break;
                    }
                }
                Servers::Local(Empty {})
            }
            ServerConf::Partial(ref list) => {
                for id in list.iter() {
                    let index = *id as usize;
                    if index >= self.conns.len() {
                        return Err(JobError::InvalidConfig(format!("server[{}] not connect;", index)));
                    }
                    if let Some(ref c) = self.conns[index] {
                        remotes.push(c);
                    } else {
                        return Err(JobError::InvalidConfig(format!("server[{}] not connect;", index)));
                    }
                }
                Servers::Part(ServerList { servers: list.clone() })
            }
            ServerConf::All => {
                for (index, conn) in self.conns.iter().enumerate() {
                    if let Some(c) = conn {
                        remotes.push(c);
                    } else {
                        return Err(JobError::InvalidConfig(format!("server[{}] not connect;", index)));
                    }
                }
                Servers::All(Empty {})
            }
        };

        let r_size = remotes.len();
        if r_size == 0 {
            warn!("No remote server selected;");
            return Ok(futures::stream::empty().boxed());
        }

        let JobDesc { input, plan, resource } = job;

        let conf = JobConfig {
            job_id: config.job_id,
            job_name: config.job_name,
            workers: config.workers,
            time_limit: config.time_limit,
            batch_size: config.batch_size,
            batch_capacity: config.batch_capacity,
            memory_limit: config.memory_limit,
            trace_enable: config.trace_enable,
            servers: Some(servers),
        };
        let req = JobRequest {
            conf: Some(conf),
            source: if input.is_empty() { None } else { Some(BinaryResource { resource: input }) },
            plan: if plan.is_empty() { None } else { Some(BinaryResource { resource: plan }) },
            resource: if resource.is_empty() { None } else { Some(BinaryResource { resource }) },
        };

        if r_size == 1 {
            match remotes[0].borrow_mut().submit(req).await {
                Ok(resp) => Ok(resp
                    .into_inner()
                    .map(|r| r.map(|jr| jr.res.map(|b| b.resource)))
                    .boxed()),
                Err(status) => Err(JobError::RPCError(status)),
            }
        } else {
            let mut tasks = Vec::with_capacity(r_size);
            for r in remotes {
                let req = req.clone();
                tasks.push(async move {
                    let mut conn = r.borrow_mut();
                    conn.submit(req).await
                })
            }
            let results = futures::future::join_all(tasks).await;
            let mut stream_res = Vec::with_capacity(results.len());
            for res in results {
                match res {
                    Ok(resp) => {
                        let stream = resp.into_inner();
                        stream_res.push(stream);
                    }
                    Err(status) => {
                        return Err(JobError::RPCError(status));
                    }
                }
            }
            Ok(futures::stream::select_all(stream_res)
                .map(|r| r.map(|jr| jr.res.map(|b| b.resource)))
                .boxed())
        }
    }
}

pub enum Either<T: Stream + Unpin> {
    Single(T),
    Select(SelectAll<T>),
}

impl<T: Stream + Unpin> Stream for Either<T> {
    type Item = T::Item;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match *self {
            Either::Single(ref mut s) => Pin::new(s).poll_next(cx),
            Either::Select(ref mut s) => Pin::new(s).poll_next(cx),
        }
    }
}