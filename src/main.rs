use structopt::StructOpt;
use rand::{Rng, thread_rng};

use computation_dag_async::computation::Computation;
use computation_dag_async::dag::Dag;
use computation_dag_async::operation::{Operation, OperationType};
use computation_dag_async::random::DagDistribution;

#[derive(Debug, StructOpt)]
#[structopt(name = "rust-dag", about = "Create directed acyclic graphs with mathematic operations.")]
struct Opt {
    /// Activate debug mode, which for printing the generated DAG and execution
    #[structopt(short, long)]
    debug: bool,

    /// Set minimum nodes in random DAG
    #[structopt(short = "n", long, default_value = "10")]
    min_nodes: u64,

    /// Set maximum nodes in random DAG
    #[structopt(short = "x", long, default_value = "50")]
    max_nodes: u64,

    /// Percentage of creating an edge from a new node to parents
    #[structopt(short = "p", long, default_value = "40")]
    edge_percentage: u32,

    /// Running mode, either "print" or "execute"
    #[structopt(short = "m", long, default_value = "print")]
    mode: String,

    /// Force DAG functions to be "delay", will always result in 0 as a response,
    /// but will show simultaneous execution
    #[structopt(long)]
    delay: bool,

    /// Force DAG functions to be "default", will always result in 0 as a response,
    /// but allows for tests with huge trees.
    #[structopt(long)]
    default: bool,
}

#[tokio::main(core_threads = 8)]
async fn main() {
    let opt = Opt::from_args();
    let min_nodes = opt.min_nodes;
    let max_nodes = opt.max_nodes;
    let edge_percentage = opt.edge_percentage;
    let default_operation = if opt.delay {
        let operation_type = OperationType::Delay;
        Some(Operation { operation_type })
    } else if opt.default {
        let operation_type = OperationType::Default;
        Some(Operation { operation_type })
    } else {
        None
    };
    let distribution = DagDistribution {
        min_nodes, max_nodes, edge_percentage, default_operation,
    };
    let mut rng = thread_rng();
    let dag: Dag = rng.sample(distribution);
    match opt.mode.as_str() {
        "print" => {
            println!("{}", dag.dot());
        },
        "execute" => {
            if opt.debug {
                println!("{}", dag.dot());
            }
            let computation = Computation::new(&dag, opt.debug);
            let initial: u128 = 1;
            let results = computation.process(initial).await;
            println!("Results: {:?}", results);
        },
        _ => panic!("Unknown mode"),
    }
}
