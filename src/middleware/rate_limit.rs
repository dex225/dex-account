use std::sync::Arc;
use tower_governor::governor::GovernorConfigBuilder;
use tower_governor::{GovernorLayer, key_extractor::PeerIpKeyExtractor};
use governor::middleware::NoOpMiddleware;

pub fn login_rate_limit() -> GovernorLayer<PeerIpKeyExtractor, NoOpMiddleware> {
    let config = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(1)
            .burst_size(5)
            .finish()
            .unwrap(),
    );
    GovernorLayer { config }
}

pub fn verify_2fa_rate_limit() -> GovernorLayer<PeerIpKeyExtractor, NoOpMiddleware> {
    let config = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(1)
            .burst_size(5)
            .finish()
            .unwrap(),
    );
    GovernorLayer { config }
}

pub fn password_forgot_rate_limit() -> GovernorLayer<PeerIpKeyExtractor, NoOpMiddleware> {
    let config = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(1)
            .burst_size(3)
            .finish()
            .unwrap(),
    );
    GovernorLayer { config }
}

pub fn general_rate_limit() -> GovernorLayer<PeerIpKeyExtractor, NoOpMiddleware> {
    let config = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(10)
            .burst_size(50)
            .finish()
            .unwrap(),
    );
    GovernorLayer { config }
}