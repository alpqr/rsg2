slotmap::new_key_type! {
    pub struct RSGNodeKey;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RSGEvent {
    SubtreeAddedOrReattached(RSGNodeKey),
    SubtreeAboutToBeRemoved(RSGNodeKey),
    SubtreeAboutToBeTemporarilyDetached(RSGNodeKey),
    Dirty(RSGNodeKey, u32)
}

pub trait RSGObserver {
    fn notify(&mut self, event: RSGEvent);
}

#[derive(Debug)]
pub enum RSGSubtreeAddOp {
    Append,
    Prepend
}

pub struct RSGSubtreeAddTransaction {
    entries: smallvec::SmallVec<[(RSGNodeKey, RSGNodeKey, RSGSubtreeAddOp); 16]>,
    #[cfg(debug_assertions)]
    possible_parent_keys: std::collections::HashSet<RSGNodeKey>
}

impl RSGSubtreeAddTransaction {
    pub fn new() -> Self {
        RSGSubtreeAddTransaction {
            entries: smallvec::SmallVec::new(),
            #[cfg(debug_assertions)]
            possible_parent_keys: std::collections::HashSet::new()
        }
    }
}

enum RSGIterState {
    AcceptAndVisitChildren(RSGNodeKey, u32),
    VisitSiblings(RSGNodeKey, u32)
}

pub struct RSGIter<'a, CompLinksT, ObserverT> where CompLinksT: Copy {
    scene: &'a RSGScene<CompLinksT, ObserverT>,
    start_key: RSGNodeKey,
    next: Option<RSGIterState>
}

impl<'a, CompLinksT, ObserverT> Iterator for RSGIter<'a, CompLinksT, ObserverT> where CompLinksT: Default + Copy, ObserverT: RSGObserver {
    type Item = (RSGNodeKey, u32);
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(state) = self.next.take() {
            match state {
                RSGIterState::AcceptAndVisitChildren(node_key, depth) => {
                    match self.scene.arena[node_key].first_child_key {
                        Some(key) => self.next = Some(RSGIterState::AcceptAndVisitChildren(key, depth + 1)),
                        None => self.next = Some(RSGIterState::VisitSiblings(node_key, depth))
                    }
                    return Some((node_key, depth));
                },
                RSGIterState::VisitSiblings(node_key, depth) if node_key != self.start_key => {
                    match self.scene.arena[node_key].next_sibling_key {
                        Some(key) => self.next = Some(RSGIterState::AcceptAndVisitChildren(key, depth)),
                        None => self.next = Some(RSGIterState::VisitSiblings(self.scene.arena[node_key].parent_key.unwrap(), depth - 1))
                    }
                },
                _ => self.next = None
            }
        }
        return None;
    }
}

pub struct RSGAncestorIter<'a, CompLinksT, ObserverT> where CompLinksT: Copy {
    scene: &'a RSGScene<CompLinksT, ObserverT>,
    next: Option<RSGNodeKey>
}

impl<'a, CompLinksT, ObserverT> Iterator for RSGAncestorIter<'a, CompLinksT, ObserverT> where CompLinksT: Default + Copy, ObserverT: RSGObserver {
    type Item = RSGNodeKey;
    fn next(&mut self) -> Option<RSGNodeKey> {
        match self.next.take() {
            Some(key) => {
                self.next = self.scene[key].parent_key;
                Some(key)
            }
            None => None
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct RSGNode<CompLinksT> where CompLinksT: Copy {
    pub key: Option<RSGNodeKey>,
    pub parent_key: Option<RSGNodeKey>,
    first_child_key: Option<RSGNodeKey>,
    last_child_key: Option<RSGNodeKey>,
    prev_sibling_key: Option<RSGNodeKey>,
    next_sibling_key: Option<RSGNodeKey>,
    comp_links: CompLinksT
}

impl<CompLinksT> RSGNode<CompLinksT> where CompLinksT: Default + Copy {
    pub fn new() -> Self {
        RSGNode {
            key: None,
            parent_key: None,
            first_child_key: None,
            last_child_key: None,
            prev_sibling_key: None,
            next_sibling_key: None,
            comp_links: Default::default()
        }
    }

    pub fn with_component_links(comp_links: CompLinksT) -> Self {
        RSGNode {
            key: None,
            parent_key: None,
            first_child_key: None,
            last_child_key: None,
            prev_sibling_key: None,
            next_sibling_key: None,
            comp_links: comp_links
        }
    }

    fn is_clean(&self) -> bool {
        self.key.is_none() && self.parent_key.is_none()
            && self.first_child_key.is_none() && self.last_child_key.is_none()
            && self.prev_sibling_key.is_none() && self.next_sibling_key.is_none()
    }

    pub fn links(&self) -> (Option<RSGNodeKey>, Option<RSGNodeKey>, Option<RSGNodeKey>, Option<RSGNodeKey>, Option<RSGNodeKey>, Option<RSGNodeKey>) {
        (self.key, self.parent_key, self.first_child_key, self.last_child_key, self.prev_sibling_key, self.next_sibling_key)
    }

    pub fn get_component_links(&self) -> &CompLinksT {
        &self.comp_links
    }

    pub fn get_component_links_mut(&mut self) -> &mut CompLinksT {
        &mut self.comp_links
    }
}

pub struct RSGScene<CompLinksT, ObserverT> where CompLinksT: Copy {
    arena: slotmap::SlotMap<RSGNodeKey, RSGNode<CompLinksT>>,
    root_key: Option<RSGNodeKey>,
    observer: Option<ObserverT>
}

impl<CompLinksT, ObserverT> RSGScene<CompLinksT, ObserverT> where CompLinksT: Default + Copy, ObserverT: RSGObserver {
    pub fn new() -> Self {
        RSGScene {
            arena: slotmap::SlotMap::with_key(),
            root_key: None,
            observer: None
        }
    }

    pub fn set_observer(&mut self, observer: ObserverT) {
        self.observer = Some(observer);
    }

    pub fn take_observer(&mut self) -> Option<ObserverT> {
        let observer = self.observer.take();
        observer
    }

    fn notify(&mut self, event: RSGEvent) {
        if let Some(obs) = self.observer.as_mut() {
            obs.notify(event);
        }
    }

    pub fn set_root(&mut self, node: RSGNode<CompLinksT>) -> RSGNodeKey {
        assert!(self.root_key.is_none());
        debug_assert!(node.is_clean());
        let key = self.arena.insert(node);
        self.root_key = Some(key);
        self.arena[key].key = self.root_key;
        self.notify(RSGEvent::SubtreeAddedOrReattached(key));
        key
    }

    pub fn root(&self) -> Option<RSGNodeKey> {
        self.root_key
    }

    pub fn node_count(&self) -> usize {
        self.arena.len()
    }

    pub fn is_valid(&self, node_key: RSGNodeKey) -> bool {
        match self.arena.get(node_key) {
            Some(node) => node.parent_key.is_some() || node.key == self.root_key,
            None => false
        }
    }

    pub fn get_component_links(&self, node_key: RSGNodeKey) -> &CompLinksT {
        self.arena.get(node_key).unwrap().get_component_links()
    }

    pub fn get_component_links_mut(&mut self, node_key: RSGNodeKey) -> &mut CompLinksT {
        self.arena.get_mut(node_key).unwrap().get_component_links_mut()
    }

    fn append_impl(&mut self, parent_key: RSGNodeKey, node_key: RSGNodeKey) {
        let old_last_node_key;
        {
            let parent_node = self.arena.get_mut(parent_key).unwrap();
            if parent_node.first_child_key.is_none() {
                parent_node.first_child_key = Some(node_key);
            }
            old_last_node_key = std::mem::replace(&mut parent_node.last_child_key, Some(node_key));
        }
        {
            let new_node = self.arena.get_mut(node_key).unwrap();
            new_node.key = Some(node_key);
            new_node.parent_key = Some(parent_key);
            new_node.prev_sibling_key = old_last_node_key;
            new_node.next_sibling_key = None; // in case links are unclean due to being called from remove_without_children()
        }
        if old_last_node_key.is_some() {
            let old_last_node = self.arena.get_mut(old_last_node_key.unwrap()).unwrap();
            debug_assert!(old_last_node.next_sibling_key.is_none());
            old_last_node.next_sibling_key = Some(node_key);
        }
    }

    pub fn append(&mut self, parent_key: RSGNodeKey, node: RSGNode<CompLinksT>) -> RSGNodeKey {
        // A(B, C) -> A(B, C, NODE)
        // Notifies: add NODE

        debug_assert!(node.is_clean() && self.is_valid(parent_key));
        let node_key = self.arena.insert(node);
        self.append_impl(parent_key, node_key);
        self.notify(RSGEvent::SubtreeAddedOrReattached(node_key));
        node_key
    }

    fn prepend_impl(&mut self, parent_key: RSGNodeKey, node_key: RSGNodeKey) {
        let old_first_node_key;
        {
            let parent_node = self.arena.get_mut(parent_key).unwrap();
            old_first_node_key = std::mem::replace(&mut parent_node.first_child_key, Some(node_key));
            if parent_node.last_child_key.is_none() {
                parent_node.last_child_key = Some(node_key);
            }
        }
        {
            let new_node = self.arena.get_mut(node_key).unwrap();
            new_node.key = Some(node_key);
            new_node.parent_key = Some(parent_key);
            new_node.prev_sibling_key = None;
            new_node.next_sibling_key = old_first_node_key;
        }
        if old_first_node_key.is_some() {
            let old_first_node = self.arena.get_mut(old_first_node_key.unwrap()).unwrap();
            debug_assert!(old_first_node.prev_sibling_key.is_none());
            old_first_node.prev_sibling_key = Some(node_key);
        }
    }

    pub fn prepend(&mut self, parent_key: RSGNodeKey, node: RSGNode<CompLinksT>) -> RSGNodeKey {
        // A(B, C) -> A(NODE, B, C)
        // Notifies: add NODE

        debug_assert!(node.is_clean() && self.is_valid(parent_key));
        let node_key = self.arena.insert(node);
        self.prepend_impl(parent_key, node_key);
        self.notify(RSGEvent::SubtreeAddedOrReattached(node_key));
        node_key
    }

    fn insert_before_impl(&mut self, before_key: RSGNodeKey, node_key: RSGNodeKey) {
        let parent_key = self.arena[before_key].parent_key;
        let old_prev_sibling_key;
        {
            let before_node = self.arena.get_mut(before_key).unwrap();
            old_prev_sibling_key = std::mem::replace(&mut before_node.prev_sibling_key, Some(node_key));
        }
        {
            let new_node = self.arena.get_mut(node_key).unwrap();
            new_node.key = Some(node_key);
            new_node.parent_key = parent_key;
            new_node.prev_sibling_key = old_prev_sibling_key;
            new_node.next_sibling_key = Some(before_key);
        }
        if old_prev_sibling_key.is_some() {
            let old_prev_node = self.arena.get_mut(old_prev_sibling_key.unwrap()).unwrap();
            old_prev_node.next_sibling_key = Some(node_key);
        } else {
            let parent_node = self.arena.get_mut(parent_key.unwrap()).unwrap();
            debug_assert!(parent_node.first_child_key == Some(before_key));
            parent_node.first_child_key = Some(node_key);
        }
    }

    pub fn insert_before(&mut self, before_key: RSGNodeKey, node: RSGNode<CompLinksT>) -> RSGNodeKey {
        // A(B, C) -> A(B, NODE, C) if before_key == C.key
        // Notifies: add NODE

        assert!(before_key != self.root_key.unwrap());
        debug_assert!(node.is_clean() && self.is_valid(before_key));
        let node_key = self.arena.insert(node);
        self.insert_before_impl(before_key, node_key);
        self.notify(RSGEvent::SubtreeAddedOrReattached(node_key));
        node_key
    }

    fn insert_after_impl(&mut self, after_key: RSGNodeKey, node_key: RSGNodeKey) {
        let parent_key = self.arena[after_key].parent_key;
        let old_next_sibling_key;
        {
            let after_node = self.arena.get_mut(after_key).unwrap();
            old_next_sibling_key = std::mem::replace(&mut after_node.next_sibling_key, Some(node_key));
        }
        {
            let new_node = self.arena.get_mut(node_key).unwrap();
            new_node.key = Some(node_key);
            new_node.parent_key = parent_key;
            new_node.prev_sibling_key = Some(after_key);
            new_node.next_sibling_key = old_next_sibling_key;
        }
        if old_next_sibling_key.is_some() {
            let old_next_node = self.arena.get_mut(old_next_sibling_key.unwrap()).unwrap();
            old_next_node.prev_sibling_key = Some(node_key);
        } else {
            let parent_node = self.arena.get_mut(parent_key.unwrap()).unwrap();
            debug_assert!(parent_node.last_child_key == Some(after_key));
            parent_node.last_child_key = Some(node_key);
        }
    }

    pub fn insert_after(&mut self, after_key: RSGNodeKey, node: RSGNode<CompLinksT>) -> RSGNodeKey {
        // A(B, C) -> A(B, NODE, C) if after_key == B.key
        // Notifies: add NODE

        assert!(after_key != self.root_key.unwrap());
        debug_assert!(node.is_clean() && self.is_valid(after_key));
        let node_key = self.arena.insert(node);
        self.insert_after_impl(after_key, node_key);
        self.notify(RSGEvent::SubtreeAddedOrReattached(node_key));
        node_key
    }

    #[inline]
    fn record_add_transaction(&mut self, op: RSGSubtreeAddOp, parent_key: RSGNodeKey, node: RSGNode<CompLinksT>, transaction: &mut RSGSubtreeAddTransaction) -> RSGNodeKey {
        debug_assert!(node.is_clean());
        debug_assert!(!transaction.entries.is_empty() || self.is_valid(parent_key));

        #[cfg(debug_assertions)]
        debug_assert!(transaction.entries.is_empty() || transaction.possible_parent_keys.contains(&parent_key));

        let node_key = self.arena.insert(node);
        transaction.entries.push((parent_key, node_key, op));

        #[cfg(debug_assertions)]
        transaction.possible_parent_keys.insert(node_key);

        node_key
    }

    pub fn append_with_transaction(&mut self, parent_key: RSGNodeKey, node: RSGNode<CompLinksT>, transaction: &mut RSGSubtreeAddTransaction) -> RSGNodeKey {
        self.record_add_transaction(RSGSubtreeAddOp::Append, parent_key, node, transaction)
    }

    pub fn prepend_with_transaction(&mut self, parent_key: RSGNodeKey, node: RSGNode<CompLinksT>, transaction: &mut RSGSubtreeAddTransaction) -> RSGNodeKey {
        self.record_add_transaction(RSGSubtreeAddOp::Prepend, parent_key, node, transaction)
    }

    pub fn commit(&mut self, transaction: RSGSubtreeAddTransaction) {
        // A(B, C) -> A(B, C, NODE(NODE2)) if transaction contains two Appends
        // (atomic subtree add: notifies only for the subtree root)
        // Notifies: add NODE

        let mut subtree_root_key_opt: Option<RSGNodeKey> = None;
        for (parent_key, node_key, op) in transaction.entries {
            match op {
                RSGSubtreeAddOp::Append => self.append_impl(parent_key, node_key),
                RSGSubtreeAddOp::Prepend => self.prepend_impl(parent_key, node_key)
            }
            if subtree_root_key_opt.is_none() {
                subtree_root_key_opt = Some(node_key);
            }
        }
        if let Some(subtree_root_key) = subtree_root_key_opt {
            self.notify(RSGEvent::SubtreeAddedOrReattached(subtree_root_key));
        }
    }

    pub fn rollback(&mut self, transaction: RSGSubtreeAddTransaction) {
        for (_, node_key, _) in transaction.entries {
            self.arena.remove(node_key);
        }
    }

    pub fn remove(&mut self, node_key: RSGNodeKey) -> CompLinksT {
        // A(NODE(B, C), D) -> A(D)
        // Notifies: remove NODE

        self.remove_helper(node_key, true)
    }

    fn remove_helper(&mut self, node_key: RSGNodeKey, with_children: bool) -> CompLinksT {
        assert!(node_key != self.root_key.unwrap());

        if with_children {
            self.notify(RSGEvent::SubtreeAboutToBeRemoved(node_key));
        } else {
            let node = self.arena.get_mut(node_key).unwrap();
            node.first_child_key = None;
            node.last_child_key = None;
            self.notify(RSGEvent::SubtreeAboutToBeRemoved(node_key));
        }

        let node = self.arena.remove(node_key).unwrap();
        let parent_key = node.parent_key.unwrap();

        if node.prev_sibling_key.is_some() && node.next_sibling_key.is_some() {
            {
                let prev_sibling_node = self.arena.get_mut(node.prev_sibling_key.unwrap()).unwrap();
                prev_sibling_node.next_sibling_key = node.next_sibling_key;
            }
            {
                let next_sibling_node = self.arena.get_mut(node.next_sibling_key.unwrap()).unwrap();
                next_sibling_node.prev_sibling_key = node.prev_sibling_key;
            }
        } else if node.prev_sibling_key.is_some() && node.next_sibling_key.is_none() {
            {
                let parent_node = self.arena.get_mut(parent_key).unwrap();
                debug_assert!(node_key == parent_node.last_child_key.unwrap());
                parent_node.last_child_key = node.prev_sibling_key;
            }
            {
                let prev_sibling_node = self.arena.get_mut(node.prev_sibling_key.unwrap()).unwrap();
                prev_sibling_node.next_sibling_key = None;
            }
        } else if node.prev_sibling_key.is_none() && node.next_sibling_key.is_some() {
            {
                let parent_node = self.arena.get_mut(parent_key).unwrap();
                debug_assert!(node_key == parent_node.first_child_key.unwrap());
                parent_node.first_child_key = node.next_sibling_key;
            }
            {
                let next_sibling_node = self.arena.get_mut(node.next_sibling_key.unwrap()).unwrap();
                next_sibling_node.prev_sibling_key = None;
            }
        } else {
            let parent_opt = self.arena.get_mut(parent_key);
            let parent_node = parent_opt.unwrap();
            debug_assert!(node_key == parent_node.first_child_key.unwrap());
            debug_assert!(node_key == parent_node.last_child_key.unwrap());
            parent_node.first_child_key = None;
            parent_node.last_child_key = None;
        }

        if with_children {
            self.remove_from_arena(node.first_child_key);
        }

        node.comp_links
    }

    fn remove_from_arena(&mut self, start_key_opt: Option<RSGNodeKey>) {
        if start_key_opt.is_none() {
            return;
        }
        let mut stk = smallvec::SmallVec::<[RSGNodeKey; 128]>::new();
        stk.push(start_key_opt.unwrap());
        while let Some(mut key) = stk.pop() {
            loop {
                let child_node = self.arena.remove(key).unwrap();
                match child_node.first_child_key {
                    Some(child_key) => stk.push(child_key),
                    None => {}
                }
                match child_node.next_sibling_key {
                    Some(sibling_key) => key = sibling_key,
                    None => break
                }
            }
        }
    }

    pub fn remove_children(&mut self, node_key: RSGNodeKey) -> smallvec::SmallVec<[CompLinksT; 16]> {
        // NODE(A, B(C)) -> NODE
        // Notifies: remove A, remove B

        let mut component_links = smallvec::smallvec![];
        let mut child_node_key_opt = self.arena[node_key].first_child_key;
        while let Some(key) = child_node_key_opt {
            child_node_key_opt = self.arena[key].next_sibling_key;
            component_links.push(self.remove(key));
        }
        component_links
    }

    pub fn clear(&mut self) -> smallvec::SmallVec<[CompLinksT; 16]> {
        match self.root_key {
            Some(key) => self.remove_children(key),
            None => smallvec::smallvec![]
        }
    }

    pub fn insert_under(&mut self, parent_key: RSGNodeKey, node: RSGNode<CompLinksT>) -> RSGNodeKey {
        // A(B, C(D)) -> A(NODE(B, C(D)))
        // Notifies: detach B, detach C, add NODE

        debug_assert!(node.is_clean() && self.is_valid(parent_key));

        let mut child_node_key_opt = self.arena[parent_key].first_child_key;
        while let Some(key) = child_node_key_opt {
            self.notify(RSGEvent::SubtreeAboutToBeTemporarilyDetached(key));
            child_node_key_opt = self.arena[key].next_sibling_key;
        }
        child_node_key_opt = self.arena[parent_key].first_child_key;

        let node_key = self.arena.insert(node);
        let mut first_child_key_opt: Option<RSGNodeKey> = Some(node_key);
        let mut last_child_key_opt: Option<RSGNodeKey> = Some(node_key);
        {
            let parent_node = self.arena.get_mut(parent_key).unwrap();
            std::mem::swap(&mut first_child_key_opt, &mut parent_node.first_child_key);
            std::mem::swap(&mut last_child_key_opt, &mut parent_node.last_child_key);
        }
        {
            let new_node = self.arena.get_mut(node_key).unwrap();
            new_node.key = Some(node_key);
            new_node.parent_key = Some(parent_key);
            new_node.first_child_key = first_child_key_opt;
            new_node.last_child_key = last_child_key_opt;
        }

        while let Some(key) = child_node_key_opt {
            self.arena.get_mut(key).unwrap().parent_key = Some(node_key);
            child_node_key_opt = self.arena[key].next_sibling_key;
        }

        self.notify(RSGEvent::SubtreeAddedOrReattached(node_key));

        node_key
    }

    pub fn remove_without_children(&mut self, node_key: RSGNodeKey) -> CompLinksT {
        // A(B, NODE(C, D(E)), F) -> A(B, C, D(E), F)
        // Notifies: detach C, detach D, remove NODE, add C, add D

        assert!(node_key != self.root_key.unwrap());
        let parent_key = self.arena[node_key].parent_key.unwrap();
        let insert_children_before_key_opt = self.arena[node_key].next_sibling_key;

        let mut child_node_key_opt = self.arena[node_key].first_child_key;
        while let Some(key) = child_node_key_opt {
            self.notify(RSGEvent::SubtreeAboutToBeTemporarilyDetached(key));
            child_node_key_opt = self.arena[key].next_sibling_key;
        }
        child_node_key_opt = self.arena[node_key].first_child_key;

        let component_links = self.remove_helper(node_key, false);

        while let Some(key) = child_node_key_opt {
            child_node_key_opt = self.arena[key].next_sibling_key;
            match insert_children_before_key_opt {
                Some(before_key) => self.insert_before_impl(before_key, key),
                None => self.append_impl(parent_key, key)
            }
            self.notify(RSGEvent::SubtreeAddedOrReattached(key));
        }

        component_links
    }

    pub fn traverse(&self, node_key: RSGNodeKey) -> RSGIter<CompLinksT, ObserverT> {
        // depth-first, pre-order
        RSGIter {
            scene: self,
            start_key: node_key,
            next: Some(RSGIterState::AcceptAndVisitChildren(node_key, 0))
        }
    }

    pub fn ancestors(&self, node_key: RSGNodeKey) -> RSGAncestorIter<CompLinksT, ObserverT> {
        // ancestors only
        RSGAncestorIter {
            scene: self,
            next: self[node_key].parent_key
        }
    }

    pub fn ancestors_with_node(&self, node_key: RSGNodeKey) -> RSGAncestorIter<CompLinksT, ObserverT> {
        // node and its ancestors
        RSGAncestorIter {
            scene: self,
            next: Some(node_key)
        }
    }

    pub fn iter(&self) -> slotmap::Iter<RSGNodeKey, RSGNode<CompLinksT>> {
        self.arena.iter()
    }

    pub fn iter_mut(&mut self) -> slotmap::IterMut<RSGNodeKey, RSGNode<CompLinksT>> {
        self.arena.iter_mut()
    }

    pub fn mark_dirty(&mut self, node_key: RSGNodeKey, flags: u32) {
        self.notify(RSGEvent::Dirty(node_key, flags));
    }
}

impl<CompLinksT, ObserverT> std::ops::Index<RSGNodeKey> for RSGScene<CompLinksT, ObserverT>
    where CompLinksT: Default + Copy, ObserverT: RSGObserver
{
    type Output = RSGNode<CompLinksT>;
    fn index(&self, node_key: RSGNodeKey) -> &Self::Output {
        self.arena.get(node_key).unwrap()
    }
}

impl<CompLinksT, ObserverT> std::ops::IndexMut<RSGNodeKey> for RSGScene<CompLinksT, ObserverT>
    where CompLinksT: Default + Copy, ObserverT: RSGObserver
{
    fn index_mut(&mut self, node_key: RSGNodeKey) -> &mut Self::Output {
        self.arena.get_mut(node_key).unwrap()
    }
}

pub type RSGSubtreeKeys = smallvec::SmallVec<[RSGNodeKey; 64]>;

pub struct RSGSubtreeBuilder<'a, CompLinksT, ObserverT> where CompLinksT: Copy {
    scene: &'a mut RSGScene<CompLinksT, ObserverT>,
    transaction: Option<RSGSubtreeAddTransaction>,
    initial_parent_key: RSGNodeKey,
    node_keys: RSGSubtreeKeys
}

impl<'a, CompLinksT, ObserverT> RSGSubtreeBuilder<'a, CompLinksT, ObserverT>
    where CompLinksT: Default + Copy, ObserverT: RSGObserver
{
    pub fn new(scene: &'a mut RSGScene<CompLinksT, ObserverT>, parent_key: RSGNodeKey) -> Self {
        RSGSubtreeBuilder {
            scene: scene,
            transaction: Some(RSGSubtreeAddTransaction::new()),
            initial_parent_key: parent_key,
            node_keys: smallvec::smallvec![]
        }
    }

    pub fn append(&mut self, node: RSGNode<CompLinksT>) -> &mut Self {
        let parent_key = self.node_keys.last().unwrap_or(&self.initial_parent_key);
        let node_key = self.scene.append_with_transaction(*parent_key, node, self.transaction.as_mut().unwrap());
        self.node_keys.push(node_key);
        self
    }

    pub fn append_to(&mut self, parent_idx: usize, node: RSGNode<CompLinksT>) -> &mut Self {
        let parent_key = self.node_keys[parent_idx];
        self.node_keys.push(self.scene.append_with_transaction(parent_key, node, self.transaction.as_mut().unwrap()));
        self
    }

    pub fn prepend(&mut self, node: RSGNode<CompLinksT>) -> &mut Self {
        let parent_key = self.node_keys.last().unwrap_or(&self.initial_parent_key);
        let node_key = self.scene.prepend_with_transaction(*parent_key, node, self.transaction.as_mut().unwrap());
        self.node_keys.push(node_key);
        self
    }

    pub fn prepend_to(&mut self, parent_idx: usize, node: RSGNode<CompLinksT>) -> &mut Self {
        let parent_key = self.node_keys[parent_idx];
        self.node_keys.push(self.scene.prepend_with_transaction(parent_key, node, self.transaction.as_mut().unwrap()));
        self
    }

    pub fn commit(&mut self) -> RSGSubtreeKeys {
        self.scene.commit(self.transaction.take().unwrap());
        std::mem::replace(&mut self.node_keys, Default::default())
    }

    pub fn rollback(&mut self) {
        self.scene.rollback(self.transaction.take().unwrap());
    }
}
