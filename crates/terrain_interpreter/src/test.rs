// Testing time
#[cfg(test)]
mod test {
    use crate::{Interpreter, NodeInterpreter};

    #[test]
    pub fn nodes() {
        // Create the interpreter system
        let mut interpreter = Interpreter::default_basic();
        let (string, _csg_tree) = interpreter.finalize().unwrap();
        let _lines = string.lines().map(|x| x.to_string()).collect::<Vec<String>>();
        println!("{}", string);
    }
}
