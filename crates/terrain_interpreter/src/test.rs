// Testing time
#[cfg(test)]
mod test {
    use crate::{nodes::*, Interpreter, NodeInterpreter};

    #[test]
    pub fn nodes() {
        // Create the interpreter system
        let mut interpreter = Interpreter::default();
        // Add the default node
        let p = BasePosition::default().new(&[], &mut interpreter).unwrap();
        // Create an snoise node
        let snoise = Noise::default().new(&[p], &mut interpreter).unwrap();
        let _snoise2 = Noise::default().new(&[p], &mut interpreter).unwrap();
        let splitter = Splitter::X.new(&[p], &mut interpreter).unwrap();
        let value = DensityOperation::Union.new(&[snoise, splitter], &mut interpreter).unwrap();
        let compare = Comparator::Equal.new(&[snoise, splitter], &mut interpreter).unwrap();
        let select = Selector().new(&[compare, snoise, value], &mut interpreter).unwrap();
        // Finalize this test interpreter with the density value of the snoise
        interpreter.finalize(select);
        let string = interpreter.read_glsl().unwrap();
        let lines = string.lines().map(|x| x.to_string()).collect::<Vec<String>>();
        println!("{}", string);
        assert!(interpreter.used_nodes.contains(&select.index));
    }
}
