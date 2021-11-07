// Testing time
#[cfg(test)]
mod test {
    use crate::{Interpreter, NodeInterpreter, error::InterpreterError, nodes::{base_position::BasePosition, comparator::Comparator, density_operations::DensityOperationType, snoise::SNoise, splitter::Splitter}};

    #[test]
    pub fn nodes() {
        // Create the interpreter system
        let mut interpreter = Interpreter::default();

        // Add the default node
        let p = BasePosition::default().new(Vec::new(), &mut interpreter).unwrap();
        // Create an snoise node
        let snoise = SNoise::default().new(vec![p], &mut interpreter).unwrap();
        let snoise2 = SNoise::default().new(vec![p], &mut interpreter).unwrap();
        let splitter = Splitter::X.new(vec![p], &mut interpreter).unwrap();
        let value = DensityOperationType::Union.new(vec![snoise, splitter], &mut interpreter).unwrap();
        let compare = Comparator::Equal.new(vec![snoise, splitter], &mut interpreter).unwrap();
        // Finalize this test interpreter with the density value of the snoise
        interpreter.finalize(compare);
        let string = interpreter.read_glsl().unwrap();
        let lines = string.lines().map(|x| x.to_string()).collect::<Vec<String>>();
        println!("{}", string);
        assert_eq!("vec3 v3_0 = pos;", lines[0]);
        assert_eq!("float d_1 = snoise(v3_0 * 0.001) * 1;", lines[1]);
    }
}