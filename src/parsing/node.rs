use crate::models::{LinkNode, TusksNode};
use syn::{Error, Item, ItemFn, ItemMod, UseTree, Visibility};

impl TusksNode {
    /// Parse a Rust module into a TusksNode tree structure
    pub fn from_module(module: &ItemMod) -> Result<Self, Error> {
        let module_name = module.ident.to_string();

        let items = module
            .content
            .as_ref()
            .map(|(_, items)| items.as_slice())
            .unwrap_or(&[]);

        let mut node = TusksNode {
            module_name,
            tusks: Vec::new(),
            childs: Vec::new(),
            links: Vec::new(),
            is_link: false,
            link_name: None
        };

        node.extract_module_items(items)?;

        Ok(node)
    }

    /// Add a child module node
    fn add_child(&mut self, module: &ItemMod) -> Result<(), Error> {
        let child_node = Self::from_module(module)?;
        self.childs.push(child_node);
        Ok(())
    }

    /// Add a link node from a use statement
    fn add_link(&mut self, name: String) {
        self.links.push(LinkNode { name });
    }

    /// Add a tusk (public function) to this node
    fn add_tusk(&mut self, func: &ItemFn) -> Result<(), Error> {
        let tusk = crate::models::Tusk::from_func(func)?;
        self.tusks.push(tusk);
        Ok(())
    }

    /// Extract all relevant items from a module
    fn extract_module_items(&mut self, items: &[Item]) -> Result<(), Error> {
        for item in items {
            match item {
                Item::Mod(submodule) if matches!(submodule.vis, Visibility::Public(_)) => {
                    self.add_child(submodule)?;
                }
                Item::Fn(func) if matches!(func.vis, Visibility::Public(_)) => {
                    // Propagate errors from add_tusk
                    self.add_tusk(func)?;
                }
                Item::Use(use_item) if matches!(use_item.vis, Visibility::Public(_)) => {
                    // Extract link nodes from 'use ...' statements
                    self.extract_use_names(&use_item.tree);
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn extract_use_names(&mut self, tree: &UseTree) {
        match tree {
            UseTree::Path(use_path) => {
                // use foo::<rest>
                self.extract_use_names(&use_path.tree);
            }
            UseTree::Name(use_name) => {
                // use foo
                self.add_link(use_name.ident.to_string());
            }
            UseTree::Rename(use_rename) => {
                // use foo as bar => take bar as name
                self.add_link(use_rename.rename.to_string());
            }
            UseTree::Glob(_) => {
                // use foo::* => ignore
            }
            UseTree::Group(use_group) => {
                // e.g. use foo::{bar, baz};
                for item in &use_group.items {
                    self.extract_use_names(item);
                }
            }
        }
    }
}
