use crate::{TusksNode, util::iterator::{links::AllLinksIter, tusks::AllTusksIter}};

impl TusksNode {
    pub fn relative_module_path(&self) -> impl Iterator<Item = &str> + '_ {
        self.module_path
            .iter()
            .skip(1)
            .map(|s| s.as_str())
    }

    /// Iterator über alle (Node, Tusk)-Paare im Baum
    pub fn iter_all_tusks(&self) -> AllTusksIter<'_> {
        AllTusksIter {
            stack: vec![self],
            current_tusks: None,
        }
    }

    /// Iterator über alle (Node, Link)-Paare
    pub fn iter_all_links(&self) -> AllLinksIter<'_> {
        AllLinksIter {
            stack: vec![self],
            current_links: None,
        }
    }
}
