fn init_spin_sleeper(limit: FrameRateLimit) -> spin_sleep::LoopHelper {
    let builder = spin_sleep::LoopHelper::builder();
    let sleeper = if let FrameRateLimit::Limited(limit) = limit {
        builder.build_with_target_rate(limit)
    } else {
        builder.build_without_target_rate()
    };

    match limit {
        FrameRateLimit::Limited(limit) => {
            log::debug!("Created sleeper with a target rate of {limit}")
        }
        FrameRateLimit::VSync => {
            log::debug!("Created sleeper without a target rate (VSync on)")
        }
        FrameRateLimit::Unlimited => {
            log::debug!("Created sleeper without a target rate (VSync off)")
        }
    }
    sleeper
}