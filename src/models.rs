use indexmap::IndexMap;

/// A tusks module or submodule which may contain of
/// - tusks (public functions in this module)
/// - childs (public submodules of this module)
/// - links (sumobudules refered to with a `pub use ...` statement)
#[derive(Debug)]
pub struct TusksNode {
    pub module_path: Vec<String>,
    pub is_link: bool,
    pub link_name: Option<String>,
    pub tusks: Vec<Tusk>,
    pub childs: Vec<TusksNode>,
    pub links: Vec<LinkNode>,
}

impl TusksNode {
    pub fn get_module_name(&self) -> &String {
        self.module_path.last().unwrap()
    }
}

/// This is just a reference to a module defined elsewhere with a `use ...` statement
#[derive(Debug)]
pub struct LinkNode {
    pub name: String,
}

/// A Tusk is essentially a public function in a tusks module.
#[derive(Debug)]
pub struct Tusk {
    pub name: String, // the function/tusk name
    pub arguments: IndexMap <String, Argument>, // argument name => argument
}

/// Essentially an argument of a function representing a tusk
/// NOTE:
/// - An argument is considered optional, if its type is Option<...>
/// - If an argument has a default value, it cannot be optional by implementation
/// This may seem a bit counterintuitive, but it simplifies the implementation and usage
#[derive(Debug)]
pub struct Argument {
    pub name: String, // name of the function argument
    pub type_: String, // rust type as string (just for cli-Docu)
    pub default: Option<String>, // default values for the argument
    pub optional: bool, // is the argument an optional argument
    pub flag: bool, // is the argument a flag (bool type)
    pub value: Option<String>, // The actual value if already set
}

pub struct AllTusksIter<'a> {
    stack: Vec<&'a TusksNode>,
    current_tusks: Option<(&'a TusksNode, std::slice::Iter<'a, Tusk>)>,
}

impl<'a> Iterator for AllTusksIter<'a> {
    type Item = (&'a TusksNode, &'a Tusk);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // Falls wir gerade in den Tusks eines Nodes iterieren
            if let Some((node, ref mut tusk_iter)) = self.current_tusks {
                if let Some(tusk) = tusk_iter.next() {
                    return Some((node, tusk));
                } else {
                    self.current_tusks = None;
                }
            }

            // Nächsten Node vom Stack holen
            let next_node = self.stack.pop()?;
            // Kinder einreihen (DFS)
            for child in next_node.childs.iter().rev() {
                self.stack.push(child);
            }

            // Iterator über seine Tusks starten
            self.current_tusks = Some((next_node, next_node.tusks.iter()));
        }
    }
}

impl TusksNode {
    /// Iterator über alle (Node, Tusk)-Paare im Baum
    pub fn iter_all_tusks(&self) -> AllTusksIter<'_> {
        AllTusksIter {
            stack: vec![self],
            current_tusks: None,
        }
    }
}

pub struct AllLinksIter<'a> {
    stack: Vec<&'a TusksNode>,
    current_links: Option<(&'a TusksNode, std::slice::Iter<'a, LinkNode>)>,
}

impl<'a> Iterator for AllLinksIter<'a> {
    type Item = (&'a TusksNode, &'a LinkNode);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // Falls wir gerade im Link-Iterator eines Nodes sind
            if let Some((node, ref mut link_iter)) = self.current_links {
                if let Some(link) = link_iter.next() {
                    return Some((node, link));
                } else {
                    self.current_links = None;
                }
            }

            // Nächsten Node vom Stack holen
            let next_node = self.stack.pop()?;
            // Kinder wieder DFS-mäßig hinzufügen
            for child in next_node.childs.iter().rev() {
                self.stack.push(child);
            }

            // Jetzt dessen Links iterieren
            self.current_links = Some((next_node, next_node.links.iter()));
        }
    }
}

impl TusksNode {
    /// Iterator über alle (Node, Link)-Paare
    pub fn iter_all_links(&self) -> AllLinksIter<'_> {
        AllLinksIter {
            stack: vec![self],
            current_links: None,
        }
    }
}
