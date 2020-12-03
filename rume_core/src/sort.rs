pub trait Sortable {
    fn next_nodes(&self, index: usize) -> Vec<usize>;
    fn num_nodes(&self) -> usize;
}

pub struct TopologicalSort<'a> {
    ordering: Vec<usize>,
    visited: Vec<bool>,
    sortable: &'a dyn Sortable,
}

impl<'a> TopologicalSort<'a> {
    fn new(sortable: &'a dyn Sortable) -> Self {
        let length = sortable.num_nodes();
        Self {
            ordering: Vec::<usize>::with_capacity(length),
            visited: vec![false; length],
            sortable,
        }
    }

    pub fn sort(sortable: &'a dyn Sortable) -> Vec<usize> {
        let mut sorter = TopologicalSort::new(sortable);

        for i in 0..sortable.num_nodes() {
            if !sorter.visited[i] {
                sorter.sort_inner(i);
            }
        }

        sorter.ordering.reverse();
        sorter.ordering
    }

    pub fn reverse_sort(sortable: &'a dyn Sortable) -> Vec<usize> {
        let mut order = Self::sort(sortable);
        order.reverse();
        order
    }

    fn sort_inner(&mut self, index: usize) {
        self.visited[index] = true;

        for i in self.sortable.next_nodes(index) {
            if !self.visited[i] {
                self.sort_inner(i);
            }
        }

        self.ordering.push(index);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::cell::RefCell;
    use std::rc::{Rc, Weak};

    #[derive(Default, Debug, Clone)]
    struct Node {
        next: Vec<Weak<RefCell<Node>>>,
    }

    impl Node {
        fn connect(&mut self, adj: Weak<RefCell<Node>>) {
            self.next.push(adj);
        }
    }

    #[derive(Default, Debug)]
    struct Graph {
        nodes: Vec<Rc<RefCell<Node>>>,
    }

    impl Graph {
        fn with_size(num_nodes: usize) -> Self {
            Self {
                nodes: (0..num_nodes)
                    .map(|_| Rc::new(RefCell::new(Node::default())))
                    .collect(),
            }
        }

        fn connect(&mut self, root: usize, next: usize) {
            self.nodes[root]
                .borrow_mut()
                .connect(Rc::downgrade(&self.nodes[next]));
        }
    }

    impl Sortable for Graph {
        fn next_nodes(&self, index: usize) -> Vec<usize> {
            let next = self.nodes.get(index).unwrap().borrow().next.clone();
            next.iter()
                .map(|node| {
                    self.nodes
                        .iter()
                        .position(|graph_node| Weak::ptr_eq(&Rc::downgrade(&graph_node), &node))
                        .unwrap()
                })
                .collect()
        }

        fn num_nodes(&self) -> usize {
            self.nodes.len()
        }
    }

    /// Validate that the test
    /// code implements Sortable
    /// as expected.
    ///
    /// [root] -> [0]
    ///      |--> [1]
    ///      |--> [2]
    ///
    #[test]
    fn test_graph_returns_adjacent_nodes() {
        const NUM_NODES: usize = 4;
        let mut graph = Graph::with_size(NUM_NODES);
        graph.connect(3, 0);
        graph.connect(3, 1);
        graph.connect(3, 2);

        for i in 0..NUM_NODES - 1 {
            assert!(graph.next_nodes(i).is_empty());
            assert!(graph.next_nodes(NUM_NODES - 1).contains(&i));
        }

        let sorted_order = TopologicalSort::sort(&graph);
        assert_eq!(sorted_order.len(), graph.nodes.len());
        assert_eq!(sorted_order[0], NUM_NODES - 1);
    }

    ///
    ///  [0] -> [1] -> [3] -> [4]
    ///    |--> [2]
    #[test]
    fn no_ordering_for_ordered_graph() {
        let mut graph = Graph::with_size(5);
        graph.connect(0, 1);
        graph.connect(0, 2);
        graph.connect(1, 3);
        graph.connect(3, 4);

        let order = TopologicalSort::sort(&graph);
        assert_eq!(order[0], 0);
        assert!([1, 2].contains(&order[1]));
        assert!([1, 2].contains(&order[2]));
        assert_eq!(order[3], 3);
        assert_eq!(order[4], 4);
    }

    ///
    ///  [7] -> [5] -> [4] -> [3] -> [0]
    ///           |--> [1] -> [6]
    ///           |--> [2]
    ///
    #[test]
    fn ordering_for_an_unordered_graph() {
        let mut graph = Graph::with_size(8);
        graph.connect(7, 5);
        graph.connect(5, 4);
        graph.connect(5, 1);
        graph.connect(5, 2);
        graph.connect(4, 3);
        graph.connect(3, 0);
        graph.connect(1, 6);

        let order = TopologicalSort::sort(&graph);
        assert_eq!(order[0], 7);
        assert_eq!(order[1], 5);
        assert_eq!(order[2], 4);
        assert_eq!(order[3], 3);
        assert_eq!(order[4], 2);
        assert_eq!(order[5], 1);
        assert_eq!(order[6], 6);
        assert_eq!(order[7], 0);
    }
}
