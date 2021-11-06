// Testing time
#[cfg(test)]
mod test {
    use crate::{Interpreter, NodeInterpreter, nodes::{base_position::BasePosition, density_operations::DensityOperationType, snoise::SNoise, splitter::Splitter}};

    #[test]
    pub fn nodes() {
        // Create the interpreter system
        let mut interpreter = Interpreter::default();

        // Add the default node
        let p = BasePosition::default().new(Vec::new(), &mut interpreter);
        // Create an snoise node
        let snoise = SNoise::default().new(vec![p], &mut interpreter);
        let snoise2 = SNoise::default().new(vec![p], &mut interpreter);
        let splitter = Splitter::X.new(vec![p], &mut interpreter);
        let value = DensityOperationType::Union.new(vec![snoise, splitter], &mut interpreter);

        // Finalize this test interpreter with the density value of the snoise
        interpreter.finalize(value);
        let hlsl = interpreter.read_hlsl().unwrap();
        let lines = hlsl.lines().map(|x| x.to_string()).collect::<Vec<String>>();
        println!("{}", hlsl);
        assert_eq!("vec3 v3_0 = pos;", lines[0]);
        assert_eq!("float d_1 = snoise(v3_0 * 0.001) * 1;", lines[1]);
    }
}