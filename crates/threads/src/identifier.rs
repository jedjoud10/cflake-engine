use crate::IterExecutionInfo;

// An ID that we can use to get what index of the vector we are currently modifying
pub struct IterExecutionID {
    // Just stores the info
    pub(crate) info: IterExecutionInfo,
}

impl IterExecutionID {
    // Get the index, and on what thread ID we are executing our iteration
    pub fn get_info(&self) -> &IterExecutionInfo {
        &self.info
    }
}
