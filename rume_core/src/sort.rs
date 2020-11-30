pub struct TopologicalSort {
    ordering: Vec<usize>,
    visited: Vec<bool>,
    adjacent: dyn Fn(usize) -> Vec<usize>,
}

impl TopologicalSort {
    pub fn sort(length: usize, adjacent: impl Fn(usize) -> Vec<usize>) -> Vec<usize> {
        let sorter = TopologicalSort {
            ordering: Vec::<usize>::with_capacity(length),
            visited: vec![false; length],
            adjacent,
        };

        for (i, was_visited) in sorter.visited.iter().enumerate() {
            if !was_visited {
                sorter.sort_inner(i);
            }
        }

        sorter.ordering.reverse();
        sorter.ordering
    }

    fn sort_inner(&mut self, index: usize) {
        self.visited[index] = true;

        for i in (self.adjacent)(index) {
            if !self.visited[i] {
                self.sort_inner(i);
            }
        }

        self.ordering.push(index);
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn no_ordering_for_ordered_graph() {}

    #[test]
    fn no_ordering_for_ordered_graph() {}
}
