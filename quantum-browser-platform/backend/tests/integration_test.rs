use qb_platform_backend::LaunchRequest;

#[test]
fn validate_launch_request() {
    let r = LaunchRequest { dashboard_id: "quantum-dashboard".into(), workspace: Some("research".into()) };
    assert!(r.validate().is_ok());

    let bad = LaunchRequest { dashboard_id: "".into(), workspace: None };
    assert!(bad.validate().is_err());
}
