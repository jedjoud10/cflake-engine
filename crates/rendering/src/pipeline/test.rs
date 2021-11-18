pub mod test {
    use crate::RenderPipeline;

    #[test]
    pub fn pipeline() {
        // Create a default pipeline and test the flow of information between the two threads
        let mut pipeline = RenderPipeline::default();
        pipeline.initialize_render_thread();
    }
}