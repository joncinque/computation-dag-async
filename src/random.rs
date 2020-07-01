use rand::distributions::{Distribution, Standard};
use rand::Rng;

use crate::dag::Dag;
use crate::operation::{Operation, OperationType};

impl Distribution<OperationType> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> OperationType {
        match rng.gen_range(0, 3) {
            0 => OperationType::Sum,
            1 => OperationType::Product,
            _ => OperationType::Default,
        }
    }
}

impl Distribution<Operation> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Operation {
        let operation_type: OperationType = rng.gen();
        Operation { operation_type }
    }
}

const MIN_NODES: u64 = 10;
const MAX_NODES: u64 = 51;
const EDGE_PERCENTAGE: u32 = 40;
const DEFAULT_OPERATION: Option<Operation> = None;

impl Distribution<Dag> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Dag {
        let min_nodes = MIN_NODES;
        let max_nodes = MAX_NODES;
        let edge_percentage = EDGE_PERCENTAGE;
        let default_operation = DEFAULT_OPERATION;
        let distribution = DagDistribution {
            min_nodes, max_nodes, edge_percentage, default_operation,
        };
        distribution.sample(rng)
    }
}

pub struct DagDistribution {
    pub min_nodes: u64,
    pub max_nodes: u64,
    pub edge_percentage: u32,
    pub default_operation: Option<Operation>,
}

impl Distribution<Dag> for DagDistribution {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Dag {
        assert!(self.min_nodes < self.max_nodes);
        assert!(0 < self.edge_percentage);
        assert!(self.edge_percentage <= 100);
        let mut dag: Dag = Default::default();
        let num_nodes: u64 = rng.gen_range(self.min_nodes, self.max_nodes);
        assert!(num_nodes > 0);
        for _ in 0..num_nodes {
            let parents = dag.nodes.iter().filter_map(|(parent_id, _)| {
                if rng.gen_ratio(self.edge_percentage, 100) {
                    Some(parent_id.clone())
                } else {
                    None
                }
            }).collect();
            let operation = match &self.default_operation {
                None => rng.gen(),
                Some(op) => op.clone(),
            };
            dag.add_node(operation, parents);
        }
        dag
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn random_operation_type() {
        let operation_type: OperationType = rand::random();
        println!("{:?}", operation_type);
    }

    #[tokio::test]
    pub async fn random_operation() {
        let operation: Operation = rand::random();
        let values = vec![1, 3, 6, 7];
        let result = operation.process(&values).await;
        match operation.operation_type {
            OperationType::Default => assert_eq!(result, 0),
            OperationType::Delay => assert_eq!(result, 0),
            OperationType::Sum => assert_eq!(result, 17),
            OperationType::Product => assert_eq!(result, 126),
        }
    }
}
