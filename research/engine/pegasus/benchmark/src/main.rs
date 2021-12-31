use std::sync::Arc;

use pegasus::JobConf;

use crate::graph::{Graph, TodoGraph};

mod graph;
mod queries;

fn main() {
    // just for compile
    let graph = load_graph::<TodoGraph>();
    let conf = JobConf::default();
    queries::ldbc::ic1(0, "Chau".to_string(), conf.clone(), graph.clone());
    queries::khop::single_src_k_hop(0, 3, true, conf.clone(), graph.clone());
    queries::khop::packed_multi_src_k_hop(vec![0], 3, true, conf.clone(), graph.clone());
    queries::khop::unpacked_multi_src_k_hop(vec![0], 3, true, conf.clone(), graph.clone());
}

fn load_graph<G: Graph>() -> Arc<G> {
    unimplemented!()
}
