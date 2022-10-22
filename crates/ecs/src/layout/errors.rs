// Error that gets returned from the query layout struct
pub enum QueryError {
    MultipleMutableAccess(&'static str),
    SimultaneousMutRefAccess(&'static str), 
    MutableAccessWhilstView(&'static str),
}


impl std::fmt::Display for QueryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self, f)
    }
}

impl std::fmt::Debug for QueryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueryError::MultipleMutableAccess(name) => write!(f, "There is duplicate mutable access to component '{name}'"),
            QueryError::MutableAccessWhilstView(name) => write!(f, "There is mutable access to component '{name}', but it is coming from an immutable archetype"),
            QueryError::SimultaneousMutRefAccess(name) => write!(f, "There is mutable access to component '{name}' whilst it is immutably accessed"),
        }
    }
}

impl std::error::Error for QueryError {}

// Error that gets returned when we try to add a bundle to an archetype
pub enum BundleError {
    DuplicateComponent(&'static str),
    MissingArchetypeTable(&'static str),
    MissingEntity,
}

impl std::fmt::Display for BundleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self, f)
    }
}

impl std::fmt::Debug for BundleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BundleError::DuplicateComponent(name) => write!(f, "There is a duplicate component '{name}' in the given bundle"),
            BundleError::MissingEntity => write!(f, "The given entity ID references a removed entity"),
            BundleError::MissingArchetypeTable(name) => write!(f, "The component table of component '{name}' does not exist on the given archetype"),
        }
    }
}

impl std::error::Error for BundleError {}