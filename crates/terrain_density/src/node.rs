// A singular node that consists of a position and an exit density
pub trait Node {
    // Get the input variable hash
    fn get_input_v_hash(&self) -> u64;
    // Get the output variable hash
    // Get the string that defines this node
    fn get_node_string(&self) -> String;
}
