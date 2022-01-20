use std::fmt::Debug;

use libloading::{Library, Symbol};
use pegasus::api::{Sink, Source};
use pegasus::result::ResultSink;
use pegasus::stream::Stream;
use pegasus::{BuildJobError, Data};

#[derive(Default)]
pub struct JobDesc {
    pub input: Vec<u8>,
    pub plan: Vec<u8>,
    pub resource: Vec<u8>,
}

impl JobDesc {
    pub fn set_input(&mut self, input_bytes: Vec<u8>) -> &mut Self {
        self.input = input_bytes;
        self
    }

    pub fn set_plan(&mut self, plan_bytes: Vec<u8>) -> &mut Self {
        self.plan = plan_bytes;
        self
    }

    pub fn set_resource(&mut self, resource_bytes: Vec<u8>) -> &mut Self {
        self.resource = plan_bytes;
        self
    }
}

pub trait JobParser<I: Data, O: Send + Debug + 'static>: Send + Sync + 'static {
    fn parse(
        &self, job: &JobDesc, input: &mut Source<I>, output: ResultSink<O>,
    ) -> Result<(), BuildJobError>;
}

pub struct DyLibParser;

impl JobParser<Vec<u8>, Vec<u8>> for DyLibParser {
    fn parse(
        &self, job: &JobDesc, input: &mut Source<Vec<u8>>, output: ResultSink<Vec<u8>>,
    ) -> Result<(), BuildJobError> {
        let stream = input.input_from(Some(job.input.clone()))?;
        let worker_index = pegasus::get_current_worker().index;
        info!("worker = {}", worker_index);
        let resource = String::from_utf8(job.resource.clone()).expect("todo");
        info!("try to load resource {} ;", resource);
        let lib = pegasus::resource::get_global_resource::<Library>(&resource).expect("todo");
        let func: Symbol<unsafe extern "Rust" fn(Stream<Vec<u8>>) -> Result<Stream<Vec<u8>>, BuildJobError>> =
            unsafe { lib.get(&job.plan[..]).expect("todo") };
        unsafe { func(stream)? }.sink_into(output)
    }
}