use crate::{LinkNode, TusksNode};

pub struct AllLinksIter<'a> {
    pub stack: Vec<&'a TusksNode>,
    pub current_links: Option<(&'a TusksNode, std::slice::Iter<'a, LinkNode>)>,
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
