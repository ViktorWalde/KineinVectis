use super::*;
use kinein_protocol::DataSourceProfile;

fn context(name: &str) -> DataSourceOperationContext {
    let profile: DataSourceProfile = serde_json::from_value(serde_json::json!({
        "name":name,"host":"localhost","port":5432,"database":"d","user":"u"}))
    .unwrap();
    DataSourceOperationContext {
        workspace: "/project".to_owned(),
        profile,
    }
}

fn request(lease: &Lease, name: &str) -> DataSourcePreviewDecideParams {
    DataSourcePreviewDecideParams {
        preview_id: lease.id().to_owned(),
        decision: DataSourcePreviewDecision::Commit,
        name: name.to_owned(),
        client_context: "1:2".to_owned(),
        expected_context: context(name),
    }
}

#[tokio::test(flavor = "current_thread")]
async fn accepts_only_ready_matching_decision_once() {
    let session = Session::default();
    let mut lease = session.reserve(context("local"), "1:2".to_owned()).unwrap();
    let mut decision = request(&lease, "local");
    assert!(session.decide(&decision).is_err());
    lease.ready("job-1").unwrap();
    assert!(lease.ready("job-1").is_err());
    decision.client_context = "old".to_owned();
    assert!(session.decide(&decision).is_err());
    decision.client_context = "1:2".to_owned();
    decision.expected_context.profile.database = "other".to_owned();
    assert!(session.decide(&decision).is_err());
    decision.expected_context = context("local");
    assert_eq!(session.decide(&decision).unwrap(), "job-1");
    assert!(session.decided_job("job-1"));
    assert!(session.decide(&decision).is_err());
    assert!(
        session
            .reserve(context("local"), "next".to_owned())
            .is_err()
    );
    session.clear(); // Does not revoke a decision already accepted.
    assert_eq!(
        lease.decision().await.unwrap(),
        DataSourcePreviewDecision::Commit
    );
}

#[test]
fn capacity_destination_and_drop_are_enforced_before_connection() {
    let session = Session::default();
    let lease = session.reserve(context("local"), "1:2".to_owned()).unwrap();
    assert!(
        session
            .reserve(context("local"), "other".to_owned())
            .is_err()
    );
    let leases: Vec<_> = (0..3)
        .map(|i| {
            session
                .reserve(context(&format!("other{i}")), "1:2".to_owned())
                .unwrap()
        })
        .collect();
    assert!(session.reserve(context("fifth"), "1:2".to_owned()).is_err());
    drop(lease);
    assert!(session.reserve(context("local"), "1:2".to_owned()).is_ok());
    drop(leases);
    assert!(session.registry.lock().unwrap().entries.is_empty());
}

#[test]
fn workspace_profile_and_expiry_discard_pending_channels() {
    let session = Session::default();
    let mut first = session.reserve(context("local"), "1:2".to_owned()).unwrap();
    let mut second = session.reserve(context("other"), "1:2".to_owned()).unwrap();
    session.revoke(std::path::Path::new("/project"), "local");
    assert!(first.discarded());
    assert!(!second.discarded());
    second.ready("job-2").unwrap();
    session
        .registry
        .lock()
        .unwrap()
        .entries
        .get_mut(second.id())
        .unwrap()
        .ready = Some((
        "job-2".to_owned(),
        Instant::now().checked_sub(Duration::from_secs(1)).unwrap(),
    ));
    assert!(session.decide(&request(&second, "other")).is_err());
    assert!(second.discarded());
    drop(first);
    let mut third = session.reserve(context("local"), "1:2".to_owned()).unwrap();
    session.clear();
    assert!(third.discarded());
}
