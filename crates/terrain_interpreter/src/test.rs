// Testing time
#[cfg(test)]
mod test {
    use crate::{nodes::*, Interpreter, NodeInterpreter};

    #[test]
    pub fn nodes() {
        // Create the interpreter system
        let mut interpreter = Interpreter::new();
        let (string, csg_tree) = interpreter.finalize().unwrap();
        let lines = string.lines().map(|x| x.to_string()).collect::<Vec<String>>();
        println!("{}", string);
    }
}
