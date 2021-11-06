// Testing time
#[cfg(test)]
mod test {
    use crate::{Interpreter, NodeInterpreter, nodes::{base_position::BasePosition, snoise::SNoise}};

    #[test]
    pub fn nodes() {
        // Create the interpreter system
        let mut interpreter = Interpreter::default();

        // Add the default node
        let p = BasePosition::default().new(Vec::new(), &mut interpreter);
        // Create an snoise node
        let snoise = SNoise::default().new(vec![p], &mut interpreter);
        let snoise2 = SNoise::default().new(vec![p], &mut interpreter);

        // Finalize this test interpreter with the density value of the snoise
        interpreter.finalize(snoise);
        println!("{}", interpreter.read_hlsl().unwrap());
    }
}