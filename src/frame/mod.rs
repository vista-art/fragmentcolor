use crate::{Pass, PassObject, Renderable};
use lsp_doc::lsp_doc;
use std::collections::VecDeque;
use std::sync::Arc;

#[cfg(python)]
use pyo3::prelude::*;
#[cfg(wasm)]
use wasm_bindgen::prelude::*;

mod features;

pub mod error;

#[cfg_attr(wasm, wasm_bindgen)]
#[cfg_attr(python, pyclass)]
#[derive(Debug, Default, Clone)]
#[lsp_doc("docs/api/core/frame/frame.md")]
pub struct Frame {
    pub(crate) passes: Vec<Arc<PassObject>>,
    dependencies: Vec<(usize, usize)>,
    present_idx: Option<usize>,
}

impl Frame {
    #[lsp_doc("docs/api/core/frame/new.md")]
    pub fn new() -> Self {
        Self {
            passes: Vec::new(),
            dependencies: Vec::new(),
            present_idx: None,
        }
    }

    #[lsp_doc("docs/api/core/frame/add_pass.md")]
    pub fn add_pass(&mut self, pass: &Pass) {
        self.passes.push(pass.object.clone());
    }

    /// Connect parent -> child (edge in the DAG)
    pub fn connect(
        &mut self,
        parent: &Pass,
        child: &Pass,
    ) -> Result<(), crate::frame::error::FrameError> {
        let p = self
            .passes
            .iter()
            .position(|x| Arc::ptr_eq(x, &parent.object))
            .ok_or(crate::frame::error::FrameError::MissingPass)?;
        let c = self
            .passes
            .iter()
            .position(|x| Arc::ptr_eq(x, &child.object))
            .ok_or(crate::frame::error::FrameError::MissingPass)?;
        if self.dependencies.iter().any(|(a, b)| *a == p && *b == c) {
            return Err(crate::frame::error::FrameError::DuplicateEdge);
        }
        self.dependencies.push((p, c));
        Ok(())
    }

    /// Select which pass presents to the final target (must be a render leaf)
    pub fn present(&mut self, pass: &Pass) -> Result<(), crate::frame::error::FrameError> {
        let idx = self
            .passes
            .iter()
            .position(|x| Arc::ptr_eq(x, &pass.object))
            .ok_or(crate::frame::error::FrameError::MissingPass)?;
        // Must be a render pass
        if pass.object.is_compute() {
            return Err(crate::frame::error::FrameError::NotRenderPass);
        }
        // Must be a leaf
        if self.dependencies.iter().any(|(a, _b)| *a == idx) {
            return Err(crate::frame::error::FrameError::NotALeaf);
        }
        // Only one present pass
        if let Some(existing) = self.present_idx {
            if existing != idx {
                return Err(crate::frame::error::FrameError::InvalidPresentPass);
            }
        }
        // Flip previous present flag off if any
        if let Some(prev) = self.present_idx.take() {
            if prev < self.passes.len() {
                if let Some(po) = self.passes.get(prev) {
                    *po.present_to_target.write() = false;
                }
            }
        }
        // Set new present target flag
        *pass.object.present_to_target.write() = true;
        self.present_idx = Some(idx);
        Ok(())
    }

    fn topo_sorted_indices(&self) -> Result<Vec<usize>, crate::frame::error::FrameError> {
        if self.dependencies.is_empty() {
            return Ok((0..self.passes.len()).collect());
        }
        let n = self.passes.len();
        let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
        let mut indeg: Vec<usize> = vec![0; n];
        for (u, v) in self.dependencies.iter().copied() {
            if u >= n || v >= n {
                continue;
            }
            adj[u].push(v);
            indeg[v] += 1;
        }
        let mut q: VecDeque<usize> = VecDeque::new();
        for i in 0..n {
            if indeg[i] == 0 {
                q.push_back(i);
            }
        }
        let mut out = Vec::with_capacity(n);
        while let Some(u) = q.pop_front() {
            out.push(u);
            for &v in &adj[u] {
                indeg[v] -= 1;
                if indeg[v] == 0 {
                    q.push_back(v);
                }
            }
        }
        if out.len() != n {
            // cycle detected
            Err(crate::frame::error::FrameError::CycleDetected)
        } else {
            Ok(out)
        }
    }
}

impl Renderable for Frame {
    fn passes(&self) -> impl IntoIterator<Item = &PassObject> {
        // Attempt topo sort; fall back to insertion order on error
        match self.topo_sorted_indices() {
            Ok(order) => order
                .into_iter()
                .map(|i| self.passes[i].as_ref())
                .collect::<Vec<_>>(),
            Err(e) => {
                log::error!("Frame::passes topo error: {}", e);
                self.passes.iter().map(|p| p.as_ref()).collect::<Vec<_>>()
            }
        }
    }
}

#[cfg(wasm)]
crate::impl_js_bridge!(Frame, crate::frame::error::FrameError);

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
