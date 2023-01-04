use crate::DynamicAttachment;

// An attachment layout is a tuple that contains multiple attachments
pub trait DynamicAttachmentLayout {
    // Get all the dynamic attachments immutably
    fn attachments(&self) -> &[&dyn DynamicAttachment];

    // Get all the dynamic attachments mutably
    fn attachments_mut(&mut self) -> &[&mut dyn DynamicAttachment];
}
