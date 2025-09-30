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
    last_pass: Option<usize>,
}

impl Frame {
    #[lsp_doc("docs/api/core/frame/new.md")]
    pub fn new() -> Self {
        Self {
            pass_indices: HashMap::new(),
            passes: Vec::new(),
            dependencies: Vec::new(),
            last_pass: None,
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

    #[lsp_doc("docs/api/core/frame/present.md")]
    pub fn present(&mut self, pass: &Pass) -> Result<(), FrameError> {
        let pass_index = self.pass_index(pass)?;

        self.validate_present_pass(pass, pass_index)?;
        self.clear_previous_present_pass();
        self.set_present_pass(pass, pass_index);

        Ok(())
    }
}

// Private methods
impl Frame {
    /// Finds the index of a pass in the frame's pass list.
    ///
    /// # Arguments
    /// * `pass` - The pass to find
    ///
    /// # Returns
    /// * `Ok(index)` if the pass is found
    /// * `Err(FrameError::MissingPass)` if the pass is not in this frame
    fn pass_index(&self, pass: &Pass) -> Result<usize, FrameError> {
        if let Some(index) = self.pass_indices.get(&pass.object.name) {
            Ok(*index)
        } else {
            Err(FrameError::PassNotFound(pass.object.name.to_string()))
        }
    }

    /// Validates that a pass can be used as a present pass.
    ///
    /// # Arguments
    /// * `pass` - The pass to validate
    /// * `pass_index` - The index of the pass in the frame
    ///
    /// # Returns
    /// * `Ok(())` if the pass is valid for presentation
    /// * `Err(FrameError::NotRenderPass)` if the pass is a compute pass
    /// * `Err(FrameError::NotALeaf)` if other passes depend on this pass
    /// * `Err(FrameError::InvalidPresentPass)` if a different pass is already presenting
    fn validate_present_pass(&self, pass: &Pass, pass_index: usize) -> Result<(), FrameError> {
        // Must be a render pass
        if pass.object.is_compute() {
            return Err(FrameError::NotRenderPass);
        }

        // Must be a leaf node (no other passes depend on it)
        if self
            .dependencies
            .iter()
            .any(|(parent_idx, _child_idx)| *parent_idx == pass_index)
        {
            return Err(FrameError::NotALeaf);
        }

        // Only one present pass allowed per frame
        if let Some(existing_present_index) = self.last_pass
            && existing_present_index != pass_index
        {
            return Err(FrameError::InvalidPresentPass);
        }

        Ok(())
    }

    /// Clears the present flag from the previously designated present pass.
    fn clear_previous_present_pass(&mut self) {
        if let Some(previous_present_index) = self.last_pass.take()
            && previous_present_index < self.passes.len()
            && let Some(previous_pass) = self.passes.get(previous_present_index)
        {
            *previous_pass.present_to_target.write() = false;
        }
    }

    /// Sets the present flag on the specified pass.
    ///
    /// # Arguments
    /// * `pass` - The pass to mark as presenting
    /// * `pass_index` - The index of the pass in the frame
    fn set_present_pass(&mut self, pass: &Pass, pass_index: usize) {
        *pass.object.present_to_target.write() = true;
        self.last_pass = Some(pass_index);
    }

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
                self.passes.iter().cloned().collect()
            }
        };
        list.into()
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
        let count = v.into_iter().count();
        assert_eq!(count, 2);
    }
}
