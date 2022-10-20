pub enum QueryValidityError {
    MultipleMutableAccess(&'static str),
    SimultaneousMutRefAccess(&'static str), 
    MutableAccessWhilstView(&'static str),
}

impl std::fmt::Display for QueryValidityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self, f)
    }
}

impl std::fmt::Debug for QueryValidityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueryValidityError::MultipleMutableAccess(_) => todo!(),
            QueryValidityError::MutableAccessWhilstView(_) => todo!(),
            QueryValidityError::SimultaneousMutRefAccess(_) => todo!(),
        }
    }
}

impl std::error::Error for QueryValidityError {}
