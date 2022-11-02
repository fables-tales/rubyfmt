      [
        Opus::Support::Command::TrackCaseAndConversation::TRACKING_LOCK,
        # rubocop:disable PrisonGuard/PrivateModule
        Opus::CIBot::Private::Model::PullRequest::TASK_LOCK,
        # rubocop:enable PrisonGuard/PrivateModule
        "mastercom_network_dispute_claim_records_nil",
        "mastercom_network_dispute_claim_record_locker_nil",
        "issuing_funding_obligation_past_due_notification",
        # rubocop:disable PrisonGuard/PrivateModule
        Opus::BankConnections::Network::Private::Command::RefreshImplicitEdgeCache::TASK_LOCK
        # rubocop:enable PrisonGuard/PrivateModule
      ]
