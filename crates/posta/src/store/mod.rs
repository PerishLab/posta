mod post;
mod read;
mod reply;
mod support;

use keel::{Core, Wire};
use std::sync::Arc;

pub struct Store<W: Wire> {
    core: Arc<Core<W>>,
}

impl<W: Wire> Clone for Store<W> {
    fn clone(&self) -> Self {
        Self {
            core: self.core.clone(),
        }
    }
}

impl<W: Wire> Store<W> {
    pub fn new(core: Arc<Core<W>>) -> Self {
        Self { core }
    }

    pub fn core(&self) -> Arc<Core<W>> {
        self.core.clone()
    }
}
