use std::ops::{Bound, RangeBounds, RangeFull};

use ahash::{AHashMap, AHashSet};

// Name key type
type Key = &'static str;

// A rule that depicts the arrangement and the location of the stages relative to other stages
#[derive(Debug, Clone, Copy)]
enum Rule {
    Before(Key),
    After(Key)
}

// Stages are are a way for us to sort and prioritize certain events before others
pub struct Stage {
    // The user defined name of the current stage
    name: Key,

    // Rules that apply to this stage
    rules: Vec<Rule>,
}

impl Stage {
    // Create a stage that will execute anytime, since it has no rules
    pub fn new(name: impl Into<Key>) -> Self {
        Self { name: name.into(), rules: Default::default()}
    }

    // Add a rule to this stage hinting it that it should execute before other
    pub fn before(mut self, other: impl Into<Key>) -> Self {
        self.rules.push(Rule::Before(other.into()));
        self
    }

    // Add a rule to this stage hinting it that it should execute after other
    pub fn after(mut self, other: impl Into<Key>) -> Self {
        self.rules.push(Rule::After(other.into()));
        self
    }

    // Free nodes are nodes that don't have any rules associated to them
    pub fn free(&self) -> bool {
        self.rules.is_empty()
    }
}


// Calculate all the priority indices of the stages and sort them automatically
fn evaluate(vec: Vec<Stage>) -> Vec<Stage> {
    // A node is each element inside the tree
    // Each node can have a parent, and some Rule that links it to it
    struct Node {
        // The user defined name for this stage (node)
        name: Key,

        // Current rules that depict how this node is connected to it's parent
        rules: Vec<Rule>,

        // The children nodes that might be connected to this node
        children: Vec<Key>,
    }

    impl Node {
        // Create a default node with a name
        fn new(name: Key) -> Self {
            Self { name, rules: Vec::new(), children: Vec::new()  }
        }
    }

    // Check if we have any duplicate stages
    let dedupped: AHashMap<Key, Stage> = AHashMap::from_iter(vec.into_iter().map(|s| (s.name, s)));
    let vec = dedupped.into_iter().map(|(_, stage)| stage).collect::<Vec<_>>();

    // We must first convert all the stages into a tree like structure with nodes
    let mut nodes: AHashMap<Key, Node> = AHashMap::new();

    // Convert all the nodes, going in arbitrary order
    for stage in vec {
        // Insert the node into the tree once, but keep updating it
        let node = nodes.entry(stage.name).or_insert_with(|| Node::new(stage.name));
        node.rules = stage.rules.clone();

        // For each rule, insert it's respective parent node
        for rule in stage.rules {
            // Get the parent name
            let parent = match rule {
                Rule::Before(p) => p,
                Rule::After(p) => p,
            };

            // Insert the parent node with placeholder data for now
            let entry = nodes.entry(parent).or_insert_with(|| Node::new(parent));
            entry.children.push(stage.name);
        }
    }  


    for (name, node) in nodes.iter() {
        dbg!(name);
        dbg!(&node.children);
        dbg!(&node.rules);
    }
    

    /*


    // This is the list of nodes that we must evaluate
    let mut eval: Vec<Key> = Vec::new();


    // A multidimensional registry to contain all the levels and nodes
    let mut levels: Vec<Vec<Key>> = Vec::new();

    // Push a new key into the levels
    fn insert_key(key: Key, idx: usize, levels: &mut Vec<Vec<Key>>) {
        if levels.len() < (idx + 1) {
            levels.push(Vec::new());
        } else {
            let level = &mut levels[idx];
            level.push(key);
        }
    }

    // Recursively evaluate the nodes
    loop {
        for name in eval.drain(..) {
            // Try to insert the node into the proper levels
            let node = &map[name];

            // Check if the node has rules
            if node.free() {
                insert_key(node.name, 0, &mut levels);

                // We must now evaluate the nodes that use the current node, aka the children
            } else {
                insert_key(node.name, )

                // We have multiple rules, so we must restrict our node
                for rule in node.rules.iter() {
                    match rule {
                        Rule::Before(other) => {
                            // Check if the current node position is on the same level as other
                            //    if it is, then make a new level before the leve of other and swap
                            // Move the node before other                            
                        },
                        Rule::After(other) => {

                        },
                    }
                }
            }

        }
    }
    */

    Vec::new()
}


#[test]
fn test() {
    let node1 = Stage::new("main").after("mycock");
    let node2 = Stage::new("substage").after("main");
    let node3 = Stage::new("sub-substage").before("main");
    evaluate(vec![node1, node2, node3]);
}