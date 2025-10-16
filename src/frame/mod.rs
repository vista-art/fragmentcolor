use crate::{Pass, PassObject, Renderable};
use lsp_doc::lsp_doc;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

#[cfg(python)]
use pyo3::prelude::*;
#[cfg(wasm)]
use wasm_bindgen::prelude::*;

mod features;

pub mod error;
pub use error::FrameError;

#[cfg_attr(wasm, wasm_bindgen)]
#[cfg_attr(python, pyclass)]
#[derive(Debug, Default, Clone)]
#[lsp_doc("docs/api/core/frame/frame.md")]
pub struct Frame {
    pub(crate) pass_indices: HashMap<Arc<str>, usize>,
    pub(crate) passes: Vec<Arc<PassObject>>,
    dependencies: Vec<(usize, usize)>,
}

crate::impl_fc_kind!(Frame, "Frame");

impl Frame {
    #[lsp_doc("docs/api/core/frame/new.md")]
    pub fn new() -> Self {
        Self {
            pass_indices: HashMap::new(),
            passes: Vec::new(),
            dependencies: Vec::new(),
        }
    }

    #[lsp_doc("docs/api/core/frame/add_pass.md")]
    pub fn add_pass(&mut self, pass: &Pass) {
        if self.pass_indices.contains_key(&pass.object.name) {
            log::warn!("Pass '{}' already exists in frame.", pass.object.name);
            return;
        }
        self.passes.push(pass.object.clone());
        self.pass_indices
            .insert(pass.object.name.clone(), self.passes.len() - 1);
    }
}

// Private methods
impl Frame {
    /// Builds an adjacency list representation of the dependency graph
    /// Returns (adjacency_list, indegree_counts)
    fn build_dependency_graph(&self) -> (Vec<Vec<usize>>, Vec<usize>) {
        let pass_count = self.passes.len();
        let mut adjacency_list: Vec<Vec<usize>> = vec![Vec::new(); pass_count];
        let mut indegree_counts: Vec<usize> = vec![0; pass_count];

        for (parent_index, child_index) in self.dependencies.iter().copied() {
            if parent_index >= pass_count || child_index >= pass_count {
                continue;
            }
            adjacency_list[parent_index].push(child_index);
            indegree_counts[child_index] += 1;
        }

        (adjacency_list, indegree_counts)
    }

    /// Performs Kahn's algorithm for topological sorting
    /// Returns Ok(sorted_indices) or Err if a cycle is detected
    fn kahn_topological_sort(
        &self,
        adjacency_list: Vec<Vec<usize>>,
        mut indegree_counts: Vec<usize>,
    ) -> Result<Vec<usize>, FrameError> {
        let pass_count = self.passes.len();
        let mut queue: VecDeque<usize> = VecDeque::new();

        // Initialize queue with nodes that have no incoming edges
        for (node_index, &indegree) in indegree_counts.iter().enumerate().take(pass_count) {
            if indegree == 0 {
                queue.push_back(node_index);
            }
        }

        let mut sorted_order = Vec::with_capacity(pass_count);

        while let Some(current_node) = queue.pop_front() {
            sorted_order.push(current_node);

            // Process all children of the current node
            for &child_node in &adjacency_list[current_node] {
                indegree_counts[child_node] -= 1;
                if indegree_counts[child_node] == 0 {
                    queue.push_back(child_node);
                }
            }
        }

        if sorted_order.len() != pass_count {
            // Cycle detected - not all nodes were processed
            Err(FrameError::CycleDetected)
        } else {
            Ok(sorted_order)
        }
    }

    /// Returns the passes in topologically sorted order based on their dependencies.
    /// If there are no dependencies, returns passes in insertion order.
    /// Returns an error if a cycle is detected in the dependency graph.
    fn topo_sorted_indices(&self) -> Result<Vec<usize>, FrameError> {
        if self.dependencies.is_empty() {
            return Ok((0..self.passes.len()).collect());
        }

        let (adjacency_list, indegree_counts) = self.build_dependency_graph();
        self.kahn_topological_sort(adjacency_list, indegree_counts)
    }
}

impl Renderable for Frame {
    /// Returns passes in this frame in topologically sorted order as Arc slice.
    fn passes(&self) -> Arc<[Arc<PassObject>]> {
        let list: Vec<Arc<PassObject>> = match self.topo_sorted_indices() {
            Ok(sorted_indices) => sorted_indices
                .into_iter()
                .map(|index| self.passes[index].clone())
                .collect(),
            Err(error) => {
                log::error!("Frame::passes topological sort error: {}", error);
                self.passes.to_vec()
            }
        };
        list.into()
    }
}

#[cfg(test)]
impl Frame {
    /// Test-only: push a dependency edge (parent_index -> child_index)
    pub(crate) fn test_add_dependency(&mut self, parent_index: usize, child_index: usize) {
        self.dependencies.push((parent_index, child_index));
    }
}

#[cfg(wasm)]
crate::impl_js_bridge!(Frame, FrameError);

#[cfg(test)]
mod tests {
    use super::*;

    // Story: A new frame starts empty, then collects passes in order, and exposes them via Renderable.
    #[test]
    fn collects_passes_and_exposes_renderable_view() {
        // Arrange
        let mut frame = Frame::new();
        let p1 = Pass::new("p1");
        let p2 = Pass::new("p2");

        // Act
        frame.add_pass(&p1);
        frame.add_pass(&p2);

        // Assert: internal storage preserves order
        assert_eq!(frame.passes.len(), 2);
        // Assert: Renderable view yields two pass objects
        let v = frame.passes();
        let count = v.iter().count();
        assert_eq!(count, 2);
    }

    // Story: topological sort orders passes per declared dependencies; cycles cause fallback.
    #[test]
    fn topological_sort_orders_or_falls_back_on_cycle() {
        // DAG case: a -> b -> c
        let mut frame = Frame::new();
        let a = Pass::new("a");
        let b = Pass::new("b");
        let c = Pass::new("c");
        frame.add_pass(&a);
        frame.add_pass(&b);
        frame.add_pass(&c);
        frame.test_add_dependency(0, 1);
        frame.test_add_dependency(1, 2);
        let names: Vec<String> = frame
            .passes()
            .iter()
            .map(|po| po.name.to_string())
            .collect();
        assert_eq!(names, vec!["a", "b", "c"]);

        // Cycle case: x <-> y
        let mut cyclic = Frame::new();
        let x = Pass::new("x");
        let y = Pass::new("y");
        cyclic.add_pass(&x);
        cyclic.add_pass(&y);
        cyclic.test_add_dependency(0, 1);
        cyclic.test_add_dependency(1, 0);
        let names_cycle: Vec<String> = cyclic
            .passes()
            .iter()
            .map(|po| po.name.to_string())
            .collect();
        // Fallback should be insertion order
        assert_eq!(names_cycle, vec!["x", "y"]);
    }
}
