use std::collections::HashMap;

use crate::operation::Operation;

pub type NodeId = u64;

pub struct Node {
    pub id: NodeId,
    pub children: Vec<NodeId>,
    pub operation: Operation,
}

impl Node {
    pub fn new(id: NodeId, operation: Operation) -> Self {
        let children = Vec::new();
        Self { id, children, operation }
    }
}

pub struct Dag {
    pub nodes: HashMap<NodeId, Node>,
    pub starts: Vec<NodeId>,
    pub current_id: NodeId,
}

impl Dag {
    pub fn next_id(&mut self) -> NodeId {
        self.current_id += 1;
        self.current_id
    }

    pub fn add_node(&mut self, operation: Operation, parents: Vec<NodeId>) -> NodeId {
        let id = self.next_id();
        let node = Node::new(id, operation);
        self.nodes.insert(id, node);
        if parents.len() == 0 {
            self.starts.push(id);
        } else {
            parents.iter().for_each(|parent_id| {
                self.nodes.get_mut(parent_id).unwrap().children.push(id)
            });
        }
        id
    }

    pub fn dot(&self) -> String {
        let mut dot = "digraph {\n".to_owned();
        self.nodes.iter().for_each(|(parent_id, node)| {
            if node.children.len() == 0 {
                dot += &format!("  {};\n", parent_id);
            } else {
                node.children.iter().for_each(|child_id| {
                    dot += &format!("  {} -> {};\n", parent_id, child_id);
                });
            }
        });
        dot += "}";
        dot
    }

}

impl Default for Dag {
    fn default() -> Self {
        let nodes = HashMap::new();
        let starts = Vec::new();
        let current_id = 0;
        Self { nodes, starts, current_id }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn add_start_nodes() {
        let mut dag: Dag = Default::default();
        let operation: Operation = Default::default();
        let id = dag.add_node(operation.clone(), vec![]);
        assert_eq!(dag.starts.len(), 1);
        assert_eq!(id, 1);
        let id = dag.add_node(operation.clone(), vec![]);
        assert_eq!(dag.starts.len(), 2);
        assert_eq!(id, 2);
    }

    #[test]
    pub fn add_children_nodes() {
        let mut dag: Dag = Default::default();
        let operation: Operation = Default::default();
        dag.add_node(operation.clone(), vec![]);
        dag.add_node(operation.clone(), vec![]);
        let id = dag.add_node(operation.clone(), vec![1, 2]);
        assert_eq!(dag.starts.len(), 2);
        assert_eq!(id, 3);
        let id = dag.add_node(operation.clone(), vec![id]);
        assert_eq!(dag.starts.len(), 2);
        assert_eq!(id, 4);
    }

    #[test]
    pub fn dot_print() {
        let mut dag: Dag = Default::default();
        let operation: Operation = Default::default();
        let id1 = dag.add_node(operation.clone(), vec![]);
        let id2 = dag.add_node(operation.clone(), vec![]);
        let id3 = dag.add_node(operation.clone(), vec![id1, id2]);
        dag.add_node(operation.clone(), vec![id3]);
        dag.add_node(operation.clone(), vec![id3]);
        let dot = dag.dot();
        assert!(dot.contains("digraph {"));
        assert!(dot.contains(" 1 -> 3;"));
        assert!(dot.contains(" 2 -> 3;"));
        assert!(dot.contains(" 3 -> 4;"));
        assert!(dot.contains(" 3 -> 5;"));
        assert!(dot.contains("}"));
    }
}
