use crate::{Tusk, TusksNode};

pub struct AllTusksIter<'a> {
    pub stack: Vec<&'a TusksNode>,
    pub current_tusks: Option<(&'a TusksNode, std::slice::Iter<'a, Tusk>)>,
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
