// Testing time
#[cfg(test)]
mod test {
    use crate::{
        nodes::{base_position::BasePosition, comparator::Comparator, density_operations::DensityOperationType, selector::Selector, snoise::SNoise, splitter::Splitter},
        Interpreter, NodeInterpreter,
    };

    #[test]
    pub fn nodes() {
        // Create the interpreter system
        let mut interpreter = Interpreter::default();
        // Add the default node
        let p = BasePosition::default().new(&[], &mut interpreter).unwrap();
        // Create an snoise node
        let snoise = SNoise::default().new(&[p], &mut interpreter).unwrap();
        let _snoise2 = SNoise::default().new(&[p], &mut interpreter).unwrap();
        let splitter = Splitter::X.new(&[p], &mut interpreter).unwrap();
        let value = DensityOperationType::Union.new(&[snoise, splitter], &mut interpreter).unwrap();
        let compare = Comparator::Equal.new(&[snoise, splitter], &mut interpreter).unwrap();
        let select = Selector().new(&[compare, snoise, value], &mut interpreter).unwrap();
        // Finalize this test interpreter with the density value of the snoise
        interpreter.finalize(select);
        let string = interpreter.read_glsl().unwrap();
        let lines = string.lines().map(|x| x.to_string()).collect::<Vec<String>>();
        println!("{}", string);
        assert_eq!("vec3 v3_0 = pos;", lines[0]);
        assert_eq!("float d_1 = snoise(v3_0 * 0.001) * 1;", lines[1]);
    }
}
