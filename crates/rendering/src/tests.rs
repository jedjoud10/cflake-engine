#[cfg(test)]
mod tests {
    use crate::{shader::Program, context::Bind};

    #[test]
    fn test() {
        let mut test: Program = todo!();
        test.bind(todo!(), |mut active| {
            let uniforms = active.uniforms();
        });
    }
}
