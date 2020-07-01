use std::collections::HashMap;
use std::thread;
use tokio::sync::oneshot::{Receiver, Sender, channel};
use futures::future::join_all;

use crate::dag::{Dag, NodeId};
use crate::operation::{Operable, Operation};

pub trait Sendable: Send + Sync {}
impl<T: Send + Sync> Sendable for T {}

pub struct ComputationNode<T>
where for<'a> T: Operable<'a, T> + Sendable + 'static {
    id: NodeId,
    operation: Operation,
    receivers: Vec<Receiver<T>>,
    senders: Vec<Sender<T>>,
    debug: bool,
}

impl<T> ComputationNode<T>
where for<'a> T: Operable<'a, T> + Sendable + 'static {
    pub fn new(id: NodeId, operation: Operation, debug: bool) -> Self {
        let receivers = vec![];
        let senders = vec![];
        ComputationNode { id, operation, receivers, senders, debug }
    }

    pub fn add_input(&mut self, receiver: Receiver<T>) {
        self.receivers.push(receiver)
    }

    pub fn add_output(&mut self, sender: Sender<T>) {
        self.senders.push(sender)
    }

    pub async fn process(mut self) {
        let mut inputs = vec![];
        for receiver in &mut self.receivers {
            let value = receiver.await.unwrap();
            inputs.push(value);
        }
        if self.debug {
            println!("{:?}: processing node {}", thread::current().id(), self.id);
        }
        let result = self.operation.process(&inputs).await;
        self.senders.into_iter().for_each(|sender| { sender.send(result.clone()).expect("Error sending"); });
    }
}

pub struct Computation<T>
where for<'a> T: Operable<'a, T> + Sendable + 'static {
    result_receivers: Vec<Receiver<T>>,
    initial_senders: Vec<Sender<T>>,
    computations: HashMap<NodeId, ComputationNode<T>>,
    debug: bool,
}

impl<T> Computation<T>
where for<'a> T: Operable<'a, T> + Sendable + 'static {
    pub fn new(dag: &Dag, debug: bool) -> Self {
        if debug {
            println!("Creating computation nodes");
        }
        let mut computations = HashMap::new();
        dag.nodes.iter().for_each(|(id, node)| {
            let computation = ComputationNode::new(id.to_owned(), node.operation.clone(), debug);
            computations.insert(computation.id, computation);
        });

        if debug {
            println!("Connecting senders and receivers");
        }
        let mut result_receivers = vec![];
        dag.nodes.iter().for_each(|(id, node)| {
            let mut parent = computations.remove(id).unwrap();
            if node.children.is_empty() {
                // Nodes with no children mean a final result, so listen from the top
                let (sender, receiver) = channel();
                parent.add_output(sender);
                result_receivers.push(receiver);
            } else {
                // Send this node's result to all children
                node.children.iter().for_each(|child_id| {
                    let child = computations.get_mut(child_id).unwrap();
                    let (sender, receiver) = channel();
                    parent.add_output(sender);
                    child.add_input(receiver);
                });
            }
            computations.insert(id.clone(), parent);
        });

        if debug {
            println!("Getting start nodes");
        }
        let mut initial_senders = vec![];
        dag.starts.iter().for_each(|id| {
            let computation = computations.get_mut(id).unwrap();
            let (sender, receiver) = channel();
            computation.add_input(receiver);
            initial_senders.push(sender);
        });

        Self { result_receivers, initial_senders, computations, debug }
    }

    pub async fn process(mut self, initial: T) -> Vec<T> {
        let mut results = vec![];

        if self.debug {
            println!("Creating tasks for node computation");
        }
        self.initial_senders.into_iter().for_each(|sender| { sender.send(initial.clone()).expect("Error sending"); });
        let tasks = self.computations.into_iter()
            .map(|(_, computation)| tokio::spawn(async move { computation.process().await }));

        if self.debug {
            println!("Starting everything!");
        }
        join_all(tasks).await;

        if self.debug {
            println!("Collecting results");
        }
        for receiver in &mut self.result_receivers {
            let value = receiver.await.unwrap();
            results.push(value);
        }

        results
    }
}

#[cfg(test)]
mod test {

    use super::*;

    use crate::operation::OperationType;

    async fn get_value() -> i32 {
        13
    }

    #[tokio::test]
    pub async fn future_split() {
        let (tx1, rx1) = channel();
        let (tx2, rx2) = channel();

        tokio::spawn(async move {
            let value = get_value().await;
            tx1.send(value).unwrap();
            tx2.send(value).unwrap();
        }).await.unwrap();

        let value1 = tokio::spawn(async move {
            rx1.await.unwrap()
        }).await.unwrap();

        let value2 = tokio::spawn(async move {
            rx2.await.unwrap()
        }).await.unwrap();
        assert_eq!(value1, value2);
    }

    #[tokio::test]
    pub async fn process_default_computation() {
        let mut dag: Dag = Default::default();
        let operation: Operation = Default::default();
        dag.add_node(operation.clone(), vec![]);
        let computation = Computation::new(&dag, false);
        let results = computation.process(0).await;
        assert_eq!(results, vec![0]);
    }

    #[tokio::test]
    pub async fn process_default_many_branches() {
        let mut dag: Dag = Default::default();
        let operation:  Operation = Default::default();
        dag.add_node(operation.clone(), vec![]);
        dag.add_node(operation.clone(), vec![]);
        dag.add_node(operation.clone(), vec![]);
        let computation = Computation::new(&dag, false);
        let results = computation.process(3).await;
        assert_eq!(results, vec![0, 0, 0]);
    }

    #[tokio::test]
    pub async fn process_addition_single_result() {
        let mut dag: Dag = Default::default();
        let operation_type = OperationType::Sum;
        let operation = Operation { operation_type };
        let id1 = dag.add_node(operation.clone(), vec![]);
        let id2 = dag.add_node(operation.clone(), vec![]);
        let id3 = dag.add_node(operation.clone(), vec![]);
        dag.add_node(operation.clone(), vec![id1, id2, id3]);
        let computation = Computation::new(&dag, false);
        let results = computation.process(3).await;
        assert_eq!(results, vec![9]);
    }

    #[tokio::test]
    pub async fn process_addition_dag() {
        let mut dag: Dag = Default::default();
        let operation_type = OperationType::Sum;
        let operation = Operation { operation_type };
        let id1 = dag.add_node(operation.clone(), vec![]);
        let id2 = dag.add_node(operation.clone(), vec![]);
        let id3 = dag.add_node(operation.clone(), vec![]);
        let id4 = dag.add_node(operation.clone(), vec![id1, id2]);
        let id5 = dag.add_node(operation.clone(), vec![id2, id3]);
        dag.add_node(operation.clone(), vec![id4, id5]);
        let computation = Computation::new(&dag, false);
        let results = computation.process(1).await;
        assert_eq!(results, vec![4]);
    }

    #[tokio::test(core_threads = 8)]
    pub async fn process_random_dag() {
        let dag: Dag = rand::random();
        let computation = Computation::new(&dag, false);
        computation.process(3).await;
    }

    #[tokio::test(core_threads = 8)]
    pub async fn process_long_dag() {
        let mut dag: Dag = Default::default();
        let operation_type = OperationType::Sum;
        let operation = Operation { operation_type };
        let mut id = dag.add_node(operation.clone(), vec![]);
        for _ in 0..100_000 {
            id = dag.add_node(operation.clone(), vec![id]);
        }
        let computation = Computation::new(&dag, false);
        computation.process(3).await;
    }

    #[tokio::test(core_threads = 8)]
    pub async fn process_wide_dag() {
        let mut dag: Dag = Default::default();
        let operation_type = OperationType::Sum;
        let operation = Operation { operation_type };
        let ids = (0..100_000).map(|_| dag.add_node(operation.clone(), vec![])).collect();
        dag.add_node(operation.clone(), ids);
        let computation = Computation::new(&dag, false);
        let results = computation.process(1).await;
        assert_eq!(results, vec![100_000]);
    }
}
