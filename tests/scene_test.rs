use rsg::scene::{RSGNode, RSGScene, RSGEvent, RSGObserver, RSGSubtreeAddTransaction, RSGSubtreeBuilder};

#[derive(Clone, Copy, Default, PartialEq)]
struct TestCompLinks {
    transform_handle: Option<usize>,
    geometry_handle: Option<usize>,
    material_handle: Option<usize>
}

struct TestObserver {
    events: Vec<RSGEvent>
}

impl RSGObserver for TestObserver {
    fn notify(&mut self, event: RSGEvent) {
        self.events.push(event);
    }
}

impl TestObserver {
    fn new() -> Self {
        TestObserver {
            events: vec![]
        }
    }
}

type TestScene = RSGScene::<TestCompLinks, TestObserver>;

#[test]
fn set_root() {
    let mut scene = TestScene::new();
    let root_key = scene.set_root(RSGNode::new());
    assert!(scene.is_valid(root_key));
    assert!(scene.node_count() == 1);
    let component_links = scene.clear();
    assert!(component_links.is_empty());
    assert!(scene.is_valid(root_key));
    assert!(scene.node_count() == 1);
}

#[test]
fn append_and_observe() {
    let mut scene = TestScene::new();
    scene.set_observer(TestObserver::new());

    // ROOT
    let root_key = scene.set_root(RSGNode::new());

    // ROOT(NODE1)
    let node1_key = scene.append(root_key, RSGNode::new());
    assert!(scene.is_valid(node1_key));
    assert!(scene.node_count() == 2);
    let mut obs = scene.take_observer().unwrap();
    assert!(obs.events.len() == 2);
    if let RSGEvent::SubtreeAddedOrReattached(key) = obs.events[0] {
        assert!(key == root_key);
    } else {
        assert!(false);
    }
    if let RSGEvent::SubtreeAddedOrReattached(key) = obs.events[1] {
        assert!(key == node1_key);
    } else {
        assert!(false);
    }

    scene.set_observer(obs);
    // ROOT(NODE1, NODE2)
    let node2_key = scene.append(root_key, RSGNode::new());
    assert!(scene.is_valid(node2_key));
    assert!(scene.node_count() == 3);
    obs = scene.take_observer().unwrap();
    assert!(obs.events.len() == 3);
    if let RSGEvent::SubtreeAddedOrReattached(key) = obs.events[2] {
        assert!(key == node2_key);
    } else {
        assert!(false);
    }

    // key, parent, first_child, last_child, prev_sibling, next_sibling
    assert!(scene[root_key].links() == (Some(root_key), None, Some(node1_key), Some(node2_key), None, None));
    assert!(scene[node1_key].links() == (Some(node1_key), Some(root_key), None, None, None, Some(node2_key)));
    assert!(scene[node2_key].links() == (Some(node2_key), Some(root_key), None, None, Some(node1_key), None));

    let component_links = scene.clear();
    assert!(component_links.len() == 2);
    assert!(scene.node_count() == 1);
    assert!(scene.is_valid(root_key));
    assert!(!scene.is_valid(node1_key));
    assert!(!scene.is_valid(node2_key));
}

#[test]
fn append_subtree_and_observe() {
    let mut scene = TestScene::new();
    // ROOT
    let root_key = scene.set_root(RSGNode::new());

    scene.set_observer(TestObserver::new());
    // ROOT(NODE1)
    let node1_key = scene.append(root_key, RSGNode::new());
    assert!(scene.is_valid(node1_key));

    let mut node2_t = RSGSubtreeAddTransaction::new();
    let node2_key = scene.append_with_transaction(root_key, RSGNode::new(), &mut node2_t);
    assert!(!scene.is_valid(node2_key));
    let node21_key = scene.append_with_transaction(node2_key, RSGNode::new(), &mut node2_t);
    assert!(!scene.is_valid(node21_key));
    let node211_key = scene.append_with_transaction(node21_key, RSGNode::new(), &mut node2_t);
    assert!(!scene.is_valid(node211_key));
    let node22_key = scene.append_with_transaction(node2_key, RSGNode::new(), &mut node2_t);
    assert!(!scene.is_valid(node22_key));

    // ROOT(NODE1, NODE2(NODE21(NODE211), NODE22))
    scene.commit(node2_t);

    assert!(scene.is_valid(node2_key));
    assert!(scene.is_valid(node21_key));
    assert!(scene.is_valid(node211_key));
    assert!(scene.is_valid(node22_key));
    assert!(scene.node_count() == 6);

    let obs = scene.take_observer().unwrap();
    assert!(obs.events.len() == 2);
    if let RSGEvent::SubtreeAddedOrReattached(key) = obs.events[0] {
        assert!(key == node1_key);
    } else {
        assert!(false);
    }
    if let RSGEvent::SubtreeAddedOrReattached(key) = obs.events[1] {
        assert!(key == node2_key);
    } else {
        assert!(false);
    }

    // key, parent, first_child, last_child, prev_sibling, next_sibling
    assert!(scene[root_key].links() == (Some(root_key), None, Some(node1_key), Some(node2_key), None, None));
    assert!(scene[node1_key].links() == (Some(node1_key), Some(root_key), None, None, None, Some(node2_key)));
    assert!(scene[node2_key].links() == (Some(node2_key), Some(root_key), Some(node21_key), Some(node22_key), Some(node1_key), None));
    assert!(scene[node21_key].links() == (Some(node21_key), Some(node2_key), Some(node211_key), Some(node211_key), None, Some(node22_key)));
    assert!(scene[node22_key].links() == (Some(node22_key), Some(node2_key), None, None, Some(node21_key), None));
    assert!(scene[node211_key].links() == (Some(node211_key), Some(node21_key), None, None, None, None));
}

#[test]
fn append_subtree_with_builder_and_observe() {
    let mut scene = TestScene::new();
    // ROOT
    let root_key = scene.set_root(RSGNode::new());

    scene.set_observer(TestObserver::new());
    // ROOT(NODE1)
    let node1_key = scene.append(root_key, RSGNode::new());
    assert!(scene.is_valid(node1_key));

    // ROOT(NODE1, NODE2(NODE21(NODE211), NODE22))
    let subtree_keys = RSGSubtreeBuilder::new(&mut scene, root_key)
    .append(RSGNode::new())
    .append(RSGNode::new())
    .append(RSGNode::new())
    .append_to(0, RSGNode::new())
    .commit();

    assert!(subtree_keys.len() == 4);
    let node2_key = subtree_keys[0];
    let node21_key = subtree_keys[1];
    let node211_key = subtree_keys[2];
    let node22_key = subtree_keys[3];

    assert!(scene.is_valid(node2_key));
    assert!(scene.is_valid(node21_key));
    assert!(scene.is_valid(node211_key));
    assert!(scene.is_valid(node22_key));
    assert!(scene.node_count() == 6);

    let obs = scene.take_observer().unwrap();
    assert!(obs.events.len() == 2);
    if let RSGEvent::SubtreeAddedOrReattached(key) = obs.events[0] {
        assert!(key == node1_key);
    } else {
        assert!(false);
    }
    if let RSGEvent::SubtreeAddedOrReattached(key) = obs.events[1] {
        assert!(key == node2_key);
    } else {
        assert!(false);
    }

    // key, parent, first_child, last_child, prev_sibling, next_sibling
    assert!(scene[root_key].links() == (Some(root_key), None, Some(node1_key), Some(node2_key), None, None));
    assert!(scene[node1_key].links() == (Some(node1_key), Some(root_key), None, None, None, Some(node2_key)));
    assert!(scene[node2_key].links() == (Some(node2_key), Some(root_key), Some(node21_key), Some(node22_key), Some(node1_key), None));
    assert!(scene[node21_key].links() == (Some(node21_key), Some(node2_key), Some(node211_key), Some(node211_key), None, Some(node22_key)));
    assert!(scene[node22_key].links() == (Some(node22_key), Some(node2_key), None, None, Some(node21_key), None));
    assert!(scene[node211_key].links() == (Some(node211_key), Some(node21_key), None, None, None, None));
}

#[test]
fn start_append_subtree_then_rollback() {
    let mut scene = TestScene::new();
    // ROOT
    let root_key = scene.set_root(RSGNode::new());

    scene.set_observer(TestObserver::new());
    // ROOT(NODE1)
    let node1_key = scene.append(root_key, RSGNode::new());
    assert!(scene.is_valid(node1_key));
    assert!(scene.node_count() == 2);

    let mut node2_t = RSGSubtreeAddTransaction::new();
    let node2_key = scene.append_with_transaction(root_key, RSGNode::new(), &mut node2_t);
    let node21_key = scene.append_with_transaction(node2_key, RSGNode::new(), &mut node2_t);
    let node211_key = scene.append_with_transaction(node21_key, RSGNode::new(), &mut node2_t);
    let node22_key = scene.append_with_transaction(node2_key, RSGNode::new(), &mut node2_t);
    // in the arena but not linked
    assert!(scene.node_count() == 6);
    assert!(scene[root_key].links() == (Some(root_key), None, Some(node1_key), Some(node1_key), None, None));

    // would have been ROOT(NODE1, NODE2(NODE21(NODE211), NODE22)) if this was a commit()
    scene.rollback(node2_t);
    // now gone from the arena too
    assert!(scene.node_count() == 2);
    assert!(!scene.is_valid(node2_key));
    assert!(!scene.is_valid(node21_key));
    assert!(!scene.is_valid(node211_key));
    assert!(!scene.is_valid(node22_key));

    let obs = scene.take_observer().unwrap();
    assert!(obs.events.len() == 1);
    if let RSGEvent::SubtreeAddedOrReattached(key) = obs.events[0] {
        assert!(key == node1_key);
    } else {
        assert!(false);
    }
}

#[test]
fn start_append_subtree_with_builder_then_rollback() {
    let mut scene = TestScene::new();
    // ROOT
    let root_key = scene.set_root(RSGNode::new());

    scene.set_observer(TestObserver::new());
    // ROOT(NODE1)
    let node1_key = scene.append(root_key, RSGNode::new());
    assert!(scene.is_valid(node1_key));
    assert!(scene.node_count() == 2);

    // ROOT(NODE1, NODE2(NODE21(NODE211), NODE22))
    RSGSubtreeBuilder::new(&mut scene, root_key)
    .append(RSGNode::new())
    .append(RSGNode::new())
    .append(RSGNode::new())
    .append_to(0, RSGNode::new())
    .rollback();

    assert!(scene.node_count() == 2);

    let obs = scene.take_observer().unwrap();
    assert!(obs.events.len() == 1);
    if let RSGEvent::SubtreeAddedOrReattached(key) = obs.events[0] {
        assert!(key == node1_key);
    } else {
        assert!(false);
    }
}

#[test]
fn prepend_and_observe() {
    let mut scene = TestScene::new();
    // ROOT
    let root_key = scene.set_root(RSGNode::new());

    scene.set_observer(TestObserver::new());
    // ROOT(NODE1)
    let node1_key = scene.prepend(root_key, RSGNode::new());
    assert!(scene.is_valid(node1_key));
    assert!(scene.node_count() == 2);
    let mut obs = scene.take_observer().unwrap();
    assert!(obs.events.len() == 1);
    if let RSGEvent::SubtreeAddedOrReattached(key) = obs.events[0] {
        assert!(key == node1_key);
    } else {
        assert!(false);
    }

    scene.set_observer(obs);
    // ROOT(NODE2, NODE1)
    let node2_key = scene.prepend(root_key, RSGNode::new());
    assert!(scene.is_valid(node2_key));
    assert!(scene.node_count() == 3);
    obs = scene.take_observer().unwrap();
    assert!(obs.events.len() == 2);
    if let RSGEvent::SubtreeAddedOrReattached(key) = obs.events[1] {
        assert!(key == node2_key);
    } else {
        assert!(false);
    }

    // key, parent, first_child, last_child, prev_sibling, next_sibling
    assert!(scene[root_key].links() == (Some(root_key), None, Some(node2_key), Some(node1_key), None, None));
    assert!(scene[node1_key].links() == (Some(node1_key), Some(root_key), None, None, Some(node2_key), None));
    assert!(scene[node2_key].links() == (Some(node2_key), Some(root_key), None, None, None, Some(node1_key)));
}

#[test]
fn prepend_subtree_with_builder_and_observe() {
    let mut scene = TestScene::new();
    // ROOT
    let root_key = scene.set_root(RSGNode::new());

    scene.set_observer(TestObserver::new());
    // ROOT(NODE1)
    let node1_key = scene.append(root_key, RSGNode::new());
    assert!(scene.is_valid(node1_key));

    // ROOT(NODE2(NODE22, NODE21(NODE211)), NODE1)
    let subtree_keys = RSGSubtreeBuilder::new(&mut scene, root_key)
    .prepend(RSGNode::new())
    .prepend(RSGNode::new())
    .prepend(RSGNode::new())
    .prepend_to(0, RSGNode::new())
    .commit();

    assert!(subtree_keys.len() == 4);
    let node2_key = subtree_keys[0];
    let node21_key = subtree_keys[1];
    let node211_key = subtree_keys[2];
    let node22_key = subtree_keys[3];

    assert!(scene.is_valid(node2_key));
    assert!(scene.is_valid(node21_key));
    assert!(scene.is_valid(node211_key));
    assert!(scene.is_valid(node22_key));
    assert!(scene.node_count() == 6);

    let obs = scene.take_observer().unwrap();
    assert!(obs.events.len() == 2);
    if let RSGEvent::SubtreeAddedOrReattached(key) = obs.events[0] {
        assert!(key == node1_key);
    } else {
        assert!(false);
    }
    if let RSGEvent::SubtreeAddedOrReattached(key) = obs.events[1] {
        assert!(key == node2_key);
    } else {
        assert!(false);
    }

    // key, parent, first_child, last_child, prev_sibling, next_sibling
    assert!(scene[root_key].links() == (Some(root_key), None, Some(node2_key), Some(node1_key), None, None));
    assert!(scene[node1_key].links() == (Some(node1_key), Some(root_key), None, None, Some(node2_key), None));
    assert!(scene[node2_key].links() == (Some(node2_key), Some(root_key), Some(node22_key), Some(node21_key), None, Some(node1_key)));
    assert!(scene[node21_key].links() == (Some(node21_key), Some(node2_key), Some(node211_key), Some(node211_key), Some(node22_key), None));
    assert!(scene[node22_key].links() == (Some(node22_key), Some(node2_key), None, None, None, Some(node21_key)));
    assert!(scene[node211_key].links() == (Some(node211_key), Some(node21_key), None, None, None, None));
}

#[test]
fn insert_before_after_and_observe() {
    let mut scene = TestScene::new();
    scene.set_observer(TestObserver::new());

    // ROOT
    let root_key = scene.set_root(RSGNode::new());

    // ROOT(NODE1)
    let node1_key = scene.append(root_key, RSGNode::new());
    // ROOT(NODE1, NODE2)
    let node2_key = scene.insert_after(node1_key, RSGNode::new());
    // ROOT(NODE1, NODE2(NODE21))
    let node21_key = scene.append(node2_key, RSGNode::new());
    // ROOT(NODE1, NODE2(NODE22, NODE21))
    let node22_key = scene.insert_before(node21_key, RSGNode::new());
    // ROOT(NODE1, NODE3, NODE2(NODE22, NODE21))
    let node3_key = scene.insert_after(node1_key, RSGNode::new());
    // ROOT(NODE4, NODE1, NODE3, NODE2(NODE22, NODE21))
    let node4_key = scene.insert_before(node1_key, RSGNode::new());

    assert!(scene.is_valid(node1_key));
    assert!(scene.is_valid(node2_key));
    assert!(scene.is_valid(node21_key));
    assert!(scene.is_valid(node22_key));
    assert!(scene.is_valid(node3_key));
    assert!(scene.is_valid(node4_key));
    assert!(scene.node_count() == 7);

    let obs = scene.take_observer().unwrap();
    assert!(obs.events.len() == 7);
    if let RSGEvent::SubtreeAddedOrReattached(key) = obs.events[1] {
        assert!(key == node1_key);
    } else {
        assert!(false);
    }
    if let RSGEvent::SubtreeAddedOrReattached(key) = obs.events[2] {
        assert!(key == node2_key);
    } else {
        assert!(false);
    }
    if let RSGEvent::SubtreeAddedOrReattached(key) = obs.events[3] {
        assert!(key == node21_key);
    } else {
        assert!(false);
    }
    if let RSGEvent::SubtreeAddedOrReattached(key) = obs.events[4] {
        assert!(key == node22_key);
    } else {
        assert!(false);
    }
    if let RSGEvent::SubtreeAddedOrReattached(key) = obs.events[5] {
        assert!(key == node3_key);
    } else {
        assert!(false);
    }
    if let RSGEvent::SubtreeAddedOrReattached(key) = obs.events[6] {
        assert!(key == node4_key);
    } else {
        assert!(false);
    }

    // key, parent, first_child, last_child, prev_sibling, next_sibling
    assert!(scene[root_key].links() == (Some(root_key), None, Some(node4_key), Some(node2_key), None, None));
    assert!(scene[node1_key].links() == (Some(node1_key), Some(root_key), None, None, Some(node4_key), Some(node3_key)));
    assert!(scene[node2_key].links() == (Some(node2_key), Some(root_key), Some(node22_key), Some(node21_key), Some(node3_key), None));
    assert!(scene[node21_key].links() == (Some(node21_key), Some(node2_key), None, None, Some(node22_key), None));
    assert!(scene[node22_key].links() == (Some(node22_key), Some(node2_key), None, None, None, Some(node21_key)));
    assert!(scene[node3_key].links() == (Some(node3_key), Some(root_key), None, None, Some(node1_key), Some(node2_key)));
    assert!(scene[node4_key].links() == (Some(node4_key), Some(root_key), None, None, None, Some(node1_key)));
}

#[test]
fn remove_and_observe() {
    let mut scene = TestScene::new();
    // ROOT
    let root_key = scene.set_root(RSGNode::new());

    scene.set_observer(TestObserver::new());
    // ROOT(NODE1)
    let node1_key = scene.append(root_key, RSGNode::new());
    assert!(scene.is_valid(node1_key));

    let mut node2_t = RSGSubtreeAddTransaction::new();
    let node2_key = scene.append_with_transaction(root_key, RSGNode::new(), &mut node2_t);
    assert!(!scene.is_valid(node2_key));
    let node21_key = scene.append_with_transaction(node2_key, RSGNode::new(), &mut node2_t);
    let node211_key = scene.append_with_transaction(node21_key, RSGNode::new(), &mut node2_t);
    let node22_key = scene.append_with_transaction(node2_key, RSGNode::new(), &mut node2_t);
    // ROOT(NODE1, NODE2(NODE21(NODE211), NODE22))
    scene.commit(node2_t);
    assert!(scene.node_count() == 6);

    let mut obs = scene.take_observer().unwrap();
    assert!(obs.events.len() == 2);

    scene.set_observer(obs);

    // ROOT(NODE1)
    scene.remove(node2_key);
    assert!(scene.node_count() == 2);
    assert!(!scene.is_valid(node2_key));
    assert!(!scene.is_valid(node21_key));
    assert!(!scene.is_valid(node211_key));
    assert!(!scene.is_valid(node22_key));
    assert!(scene[root_key].links() == (Some(root_key), None, Some(node1_key), Some(node1_key), None, None));

    obs = scene.take_observer().unwrap();
    assert!(obs.events.len() == 3);
    if let RSGEvent::SubtreeAboutToBeRemoved(key) = obs.events[2] {
        assert!(key == node2_key);
    } else {
        assert!(false);
    }

    scene.set_observer(obs);
    // ROOT
    let component_links = scene.remove_children(root_key);
    assert!(component_links.len() == 1);
    assert!(scene.node_count() == 1);

    obs = scene.take_observer().unwrap();
    assert!(obs.events.len() == 4);
    if let RSGEvent::SubtreeAboutToBeRemoved(key) = obs.events[3] {
        assert!(key == node1_key);
    } else {
        assert!(false);
    }
}

#[test]
fn remove_single_child_without_children_and_observe()
{
    let mut scene = TestScene::new();
    // ROOT
    let root_key = scene.set_root(RSGNode::new());

    // ROOT(NODE1)
    let node1_key = scene.append(root_key, RSGNode::new());
    // ROOT(NODE1, NODE2)
    let node2_key = scene.append(root_key, RSGNode::new());
    // ROOT(NODE1(NODE11), NODE2)
    let node11_key = scene.append(node1_key, RSGNode::new());
    // ROOT(NODE1(NODE11(NODE111), NODE2)
    let node111_key = scene.append(node11_key, RSGNode::new());
    // ROOT(NODE1(NODE11(NODE111, NODE112)), NODE2)
    let node112_key = scene.append(node11_key, RSGNode::new());

    assert!(scene.node_count() == 6);
    {
        // key, parent, first_child, last_child, prev_sibling, next_sibling
        assert!(scene[node1_key].links() == (Some(node1_key), Some(root_key), Some(node11_key), Some(node11_key), None, Some(node2_key)));
        assert!(scene[node11_key].links() == (Some(node11_key), Some(node1_key), Some(node111_key), Some(node112_key), None, None));
        assert!(scene[node111_key].links() == (Some(node111_key), Some(node11_key), None, None, None, Some(node112_key)));
        assert!(scene[node112_key].links() == (Some(node112_key), Some(node11_key), None, None, Some(node111_key), None));
    }

    scene.set_observer(TestObserver::new());

    // ROOT(NODE1(NODE111, NODE112), NODE2)
    scene.remove_without_children(node11_key);

    assert!(scene.node_count() == 5);
    assert!(!scene.is_valid(node11_key));
    assert!(scene.is_valid(node111_key));
    assert!(scene.is_valid(node112_key));

    {
        assert!(scene[node1_key].links() == (Some(node1_key), Some(root_key), Some(node111_key), Some(node112_key), None, Some(node2_key)));
        assert!(scene[node111_key].links() == (Some(node111_key), Some(node1_key), None, None, None, Some(node112_key)));
        assert!(scene[node112_key].links() == (Some(node112_key), Some(node1_key), None, None, Some(node111_key), None));
    }

    let obs = scene.take_observer().unwrap();
    assert!(obs.events.len() == 5);
    if let RSGEvent::SubtreeAboutToBeTemporarilyDetached(key) = obs.events[0] {
        assert!(key == node111_key);
    } else {
        assert!(false);
    }
    if let RSGEvent::SubtreeAboutToBeTemporarilyDetached(key) = obs.events[1] {
        assert!(key == node112_key);
    } else {
        assert!(false);
    }
    if let RSGEvent::SubtreeAboutToBeRemoved(key) = obs.events[2] {
        assert!(key == node11_key);
    } else {
        assert!(false);
    }
    if let RSGEvent::SubtreeAddedOrReattached(key) = obs.events[3] {
        assert!(key == node111_key);
    } else {
        assert!(false);
    }
    if let RSGEvent::SubtreeAddedOrReattached(key) = obs.events[4] {
        assert!(key == node112_key);
    } else {
        assert!(false);
    }

    // ROOT(NODE1(NODE111, NODE112), NODE2(NODE21))
    let node21_key = scene.append(node2_key, RSGNode::new());
    assert!(scene.node_count() == 6);
    // ROOT(NODE1(NODE111, NODE112), NODE21)
    scene.remove_without_children(node2_key);
    assert!(scene.node_count() == 5);
    assert!(!scene.is_valid(node2_key));
    assert!(scene.is_valid(node21_key));
    {
        assert!(scene[root_key].links() == (Some(root_key), None, Some(node1_key), Some(node21_key), None, None));
        assert!(scene[node1_key].links() == (Some(node1_key), Some(root_key), Some(node111_key), Some(node112_key), None, Some(node21_key)));
        assert!(scene[node21_key].links() == (Some(node21_key), Some(root_key), None, None, Some(node1_key), None));
    }
}

#[test]
fn remove_without_children_and_observe()
{
    let mut scene = TestScene::new();
    // ROOT
    let root_key = scene.set_root(RSGNode::new());

    // ROOT(NODE1)
    let node1_key = scene.append(root_key, RSGNode::new());
    // ROOT(NODE1, NODE2)
    let node2_key = scene.append(root_key, RSGNode::new());
    // ROOT(NODE1, NODE2(NODE21))
    let node21_key = scene.append(node2_key, RSGNode::new());
    // ROOT(NODE1, NODE2(NODE21, NODE22))
    let node22_key = scene.append(node2_key, RSGNode::new());
    // ROOT(NODE1, NODE2(NODE21, NODE22(NODE221)))
    let node221_key = scene.append(node22_key, RSGNode::new());
    // ROOT(NODE1, NODE2(NODE21, NODE22(NODE221)), NODE3)
    let node3_key = scene.append(root_key, RSGNode::new());
    assert!(scene.node_count() == 7);
    scene.set_observer(TestObserver::new());

    // ROOT(NODE1, NODE21, NODE22(NODE221), NODE3)
    scene.remove_without_children(node2_key);

    assert!(scene.node_count() == 6);
    assert!(scene.is_valid(node1_key));
    assert!(!scene.is_valid(node2_key));
    assert!(scene.is_valid(node21_key));
    assert!(scene.is_valid(node22_key));
    assert!(scene.is_valid(node221_key));
    assert!(scene.is_valid(node3_key));

    {
        // key, parent, first_child, last_child, prev_sibling, next_sibling
        assert!(scene[root_key].links() == (Some(root_key), None, Some(node1_key), Some(node3_key), None, None));
        assert!(scene[node1_key].links() == (Some(node1_key), Some(root_key), None, None, None, Some(node21_key)));
        assert!(scene[node21_key].links() == (Some(node21_key), Some(root_key), None, None, Some(node1_key), Some(node22_key)));
        assert!(scene[node22_key].links() == (Some(node22_key), Some(root_key), Some(node221_key), Some(node221_key), Some(node21_key), Some(node3_key)));
        assert!(scene[node221_key].links() == (Some(node221_key), Some(node22_key), None, None, None, None));
        assert!(scene[node3_key].links() == (Some(node3_key), Some(root_key), None, None, Some(node22_key), None));
    }

    let obs = scene.take_observer().unwrap();
    assert!(obs.events.len() == 5);
    if let RSGEvent::SubtreeAboutToBeTemporarilyDetached(key) = obs.events[0] {
        assert!(key == node21_key);
    } else {
        assert!(false);
    }
    if let RSGEvent::SubtreeAboutToBeTemporarilyDetached(key) = obs.events[1] {
        assert!(key == node22_key);
    } else {
        assert!(false);
    }
    if let RSGEvent::SubtreeAboutToBeRemoved(key) = obs.events[2] {
        assert!(key == node2_key);
    } else {
        assert!(false);
    }
    if let RSGEvent::SubtreeAddedOrReattached(key) = obs.events[3] {
        assert!(key == node21_key);
    } else {
        assert!(false);
    }
    if let RSGEvent::SubtreeAddedOrReattached(key) = obs.events[4] {
        assert!(key == node22_key);
    } else {
        assert!(false);
    }
}

#[test]
fn insert_under_and_observe()
{
    let mut scene = TestScene::new();
    // ROOT
    let root_key = scene.set_root(RSGNode::new());

    // ROOT(NODE1)
    let node1_key = scene.append(root_key, RSGNode::new());
    // ROOT(NODE1, NODE2)
    let node2_key = scene.append(root_key, RSGNode::new());

    scene.set_observer(TestObserver::new());

    // ROOT(NODE3(NODE1, NODE2))
    let node3_key = scene.insert_under(root_key, RSGNode::new());

    {
        // key, parent, first_child, last_child, prev_sibling, next_sibling
        assert!(scene[root_key].links() == (Some(root_key), None, Some(node3_key), Some(node3_key), None, None));
        assert!(scene[node3_key].links() == (Some(node3_key), Some(root_key), Some(node1_key), Some(node2_key), None, None));
        assert!(scene[node1_key].links() == (Some(node1_key), Some(node3_key), None, None, None, Some(node2_key)));
        assert!(scene[node2_key].links() == (Some(node2_key), Some(node3_key), None, None, Some(node1_key), None));
    }

    let mut obs = scene.take_observer().unwrap();
    assert!(obs.events.len() == 3);
    if let RSGEvent::SubtreeAboutToBeTemporarilyDetached(key) = obs.events[0] {
        assert!(key == node1_key);
    } else {
        assert!(false);
    }
    if let RSGEvent::SubtreeAboutToBeTemporarilyDetached(key) = obs.events[1] {
        assert!(key == node2_key);
    } else {
        assert!(false);
    }
    if let RSGEvent::SubtreeAddedOrReattached(key) = obs.events[2] {
        assert!(key == node3_key);
    } else {
        assert!(false);
    }
    obs.events.clear();
    scene.set_observer(obs);

    // ROOT(NODE3(NODE1, NODE2), NODE4)
    let node4_key = scene.append(root_key, RSGNode::new());
    // ROOT(NODE3(NODE5(NODE1, NODE2)), NODE4)
    let node5_key = scene.insert_under(node3_key, RSGNode::new());

    {
        assert!(scene[node1_key].links() == (Some(node1_key), Some(node5_key), None, None, None, Some(node2_key)));
        assert!(scene[node2_key].links() == (Some(node2_key), Some(node5_key), None, None, Some(node1_key), None));
        assert!(scene[node3_key].links() == (Some(node3_key), Some(root_key), Some(node5_key), Some(node5_key), None, Some(node4_key)));
        assert!(scene[node4_key].links() == (Some(node4_key), Some(root_key), None, None, Some(node3_key), None));
        assert!(scene[node5_key].links() == (Some(node5_key), Some(node3_key), Some(node1_key), Some(node2_key), None, None));
    }

    obs = scene.take_observer().unwrap();
    assert!(obs.events.len() == 4);
    if let RSGEvent::SubtreeAddedOrReattached(key) = obs.events[0] {
        assert!(key == node4_key);
    } else {
        assert!(false);
    }
    if let RSGEvent::SubtreeAboutToBeTemporarilyDetached(key) = obs.events[1] {
        assert!(key == node1_key);
    } else {
        assert!(false);
    }
    if let RSGEvent::SubtreeAboutToBeTemporarilyDetached(key) = obs.events[2] {
        assert!(key == node2_key);
    } else {
        assert!(false);
    }
    if let RSGEvent::SubtreeAddedOrReattached(key) = obs.events[3] {
        assert!(key == node5_key);
    } else {
        assert!(false);
    }
}

#[test]
fn traversal() {
    let mut scene = TestScene::new();
    let root_key = scene.set_root(RSGNode::new());
    for (node_key, depth) in scene.traverse(root_key) {
        assert!(node_key == root_key);
        assert!(depth == 0);
    }

    // ROOT(NODE1(NODE11, NODE12), NODE2(NODE21))
    let node1_key = scene.append(root_key, RSGNode::new());
    for (node_key, depth) in scene.traverse(root_key) {
        match depth {
            0 => assert!(node_key == root_key),
            1 => assert!(node_key == node1_key),
            _ => assert!(false)
        }
    }
    for (node_key, depth) in scene.traverse(node1_key) {
        match depth {
            0 => assert!(node_key == node1_key),
            _ => assert!(false)
        }
    }
    let node2_key = scene.append(root_key, RSGNode::new());
    let node11_key = scene.append(node1_key, RSGNode::new());
    let node12_key = scene.append(node1_key, RSGNode::new());
    let node21_key = scene.append(node2_key, RSGNode::new());

    let mut n = 0;
    let expected = [root_key, node1_key, node11_key, node12_key, node2_key, node21_key];
    let expected_depth = [0, 1, 2, 2, 1, 2];
    for (node_key, depth) in scene.traverse(root_key) {
        assert!(node_key == expected[n]);
        assert!(depth == expected_depth[n]);
        n += 1;
    }

    let mut n = 0;
    let expected = [node1_key, node11_key, node12_key];
    let expected_depth = [0, 1, 1];
    for (node_key, depth) in scene.traverse(node1_key) {
        assert!(node_key == expected[n]);
        assert!(depth == expected_depth[n]);
        n += 1;
    }

    let mut n = 0;
    let expected = [node2_key, node21_key];
    let expected_depth = [0, 1, 1];
    for (node_key, depth) in scene.traverse(node2_key) {
        assert!(node_key == expected[n]);
        assert!(depth == expected_depth[n]);
        n += 1;
    }

    let expected = [root_key, node1_key, node2_key, node11_key, node12_key, node21_key];
    for (i, (k, _)) in scene.iter().enumerate() {
        assert!(expected[i] == k);
    }
}

#[test]
fn visit_ancestors() {
    let mut scene = TestScene::new();
    // ROOT(NODE1(NODE11, NODE12), NODE2(NODE21(NODE22)))
    let root_key = scene.set_root(RSGNode::new());
    let node1_key = scene.append(root_key, RSGNode::new());
    let node2_key = scene.append(root_key, RSGNode::new());
    let _node11_key = scene.append(node1_key, RSGNode::new());
    let node12_key = scene.append(node1_key, RSGNode::new());
    let node21_key = scene.append(node2_key, RSGNode::new());
    let node22_key = scene.append(node21_key, RSGNode::new());

    for node_key in scene.ancestors_with_node(root_key) {
        assert!(node_key == root_key);
    }

    for _ in scene.ancestors(root_key) {
        assert!(false);
    }

    let mut n = 0;
    let expected = [node12_key, node1_key, root_key];
    for node_key in scene.ancestors_with_node(node12_key) {
        assert!(node_key == expected[n]);
        n += 1;
    }

    let mut n = 0;
    let expected = [node1_key, root_key];
    for node_key in scene.ancestors(node12_key) {
        assert!(node_key == expected[n]);
        n += 1;
    }

    let mut parent_key = Some(node22_key);
    for node_key in scene.ancestors_with_node(node22_key) {
        assert!(node_key == parent_key.unwrap());
        parent_key = scene[node_key].parent_key;
    }
}

#[test]
fn component_links() {
    let mut scene = TestScene::new();
    let root_key = scene.set_root(RSGNode::new());
    let node1_key = scene.append(root_key, RSGNode::new());
    let _node2_key = scene.append(root_key, RSGNode::new());
    // ROOT(NODE1, NODE2)

    {
        let c: &TestCompLinks = scene[node1_key].get_component_links();
        assert!(c.transform_handle.is_none());
        assert!(c.geometry_handle.is_none());
        assert!(c.material_handle.is_none());
    }
    {
        let c: &mut TestCompLinks = scene[node1_key].get_component_links_mut();
        c.transform_handle = Some(123);
        c.geometry_handle = Some(456);
        c.material_handle = Some(789);
    }

    let _node3_key = scene.prepend(root_key, RSGNode::new());
    // ROOT(NODE3, NODE1, NODE2)

    {
        let c: &TestCompLinks = scene[node1_key].get_component_links();
        assert!(c.transform_handle == Some(123));
        assert!(c.geometry_handle == Some(456));
        assert!(c.material_handle == Some(789));
        let c2: &TestCompLinks = scene.get_component_links(node1_key);
        assert!(c == c2);
    }
}

#[test]
fn remove_and_add_new_with_same_component_links() {
    let mut scene = TestScene::new();
    // ROOT(NODE1, NODE2)
    let root_key = scene.set_root(RSGNode::new());
    let c = TestCompLinks {
        transform_handle: Some(1),
        geometry_handle: Some(2),
        material_handle: Some(3)
    };
    let node1_key = scene.append(root_key, RSGNode::with_component_links(c));
    let c = TestCompLinks {
        transform_handle: Some(4),
        geometry_handle: Some(5),
        material_handle: Some(6)
    };
    let node2_key = scene.append(root_key, RSGNode::with_component_links(c));

    assert!(scene.node_count() == 3);
    assert!(scene.get_component_links(node1_key).transform_handle == Some(1));
    assert!(scene.get_component_links(node2_key).transform_handle == Some(4));

    // ROOT(NODE2)
    let c = scene.remove(node1_key);
    assert!(scene.node_count() == 2);
    assert!(!scene.is_valid(node1_key));
    assert!(c.transform_handle == Some(1));

    // ROOT(NODE2, NODE3)
    let node3_key = scene.append(root_key, RSGNode::with_component_links(c));

    assert!(scene.node_count() == 3);
    assert!(!scene.is_valid(node1_key));
    assert!(scene.is_valid(node2_key));
    assert!(scene.is_valid(node3_key));
    assert!(scene.get_component_links(node3_key).transform_handle == Some(1));
    assert!(scene.get_component_links(node3_key).geometry_handle == Some(2));
    assert!(scene.get_component_links(node3_key).material_handle == Some(3));

    assert!(scene.get_component_links(node2_key).transform_handle == Some(4));
    assert!(scene.get_component_links(node2_key).geometry_handle == Some(5));
    assert!(scene.get_component_links(node2_key).material_handle == Some(6));

    // key, parent, first_child, last_child, prev_sibling, next_sibling
    assert!(scene[root_key].links() == (Some(root_key), None, Some(node2_key), Some(node3_key), None, None));
    assert!(scene[node2_key].links() == (Some(node2_key), Some(root_key), None, None, None, Some(node3_key)));
    assert!(scene[node3_key].links() == (Some(node3_key), Some(root_key), None, None, Some(node2_key), None));
}

#[test]
fn mark_dirty() {
    let mut scene = TestScene::new();

    // ROOT(NODE1)
    let root_key = scene.set_root(RSGNode::new());
    let node1_key = scene.append(root_key, RSGNode::new());

    scene.set_observer(TestObserver::new());
    scene.mark_dirty(node1_key, 123);
    let obs = scene.take_observer().unwrap();
    assert!(obs.events.len() == 1);
    if let RSGEvent::Dirty(key, flags) = obs.events[0] {
        assert!(key == node1_key && flags == 123);
    } else {
        assert!(false);
    }
}
