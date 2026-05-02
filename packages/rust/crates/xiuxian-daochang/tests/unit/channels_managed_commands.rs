//! Test coverage for xiuxian-daochang behavior.

use xiuxian_daochang::test_support::{
    ManagedControlCommand, ManagedSlashCommand, OutputFormat, ResumeContextCommand,
    SessionAdminAction, SessionFeedbackDirection, SessionInjectionAction, SessionMentionMode,
    SessionPartitionMode, detect_managed_control_command, detect_managed_slash_command,
    test_is_agenda_command, test_parse_help_command, test_parse_job_status_command,
    test_parse_resume_context_command, test_parse_session_admin_command,
    test_parse_session_feedback_command, test_parse_session_injection_command,
    test_parse_session_mention_command, test_parse_session_partition_command,
};

#[test]
fn test_support_parses_help_and_job_status_output_formats() {
    assert_eq!(
        test_parse_help_command("/help"),
        Some(OutputFormat::Dashboard)
    );
    assert_eq!(
        test_parse_help_command("/help json"),
        Some(OutputFormat::Json)
    );
    assert!(test_is_agenda_command("/agenda"));
    assert!(test_is_agenda_command("agenda"));
    assert!(!test_is_agenda_command("/agenda tomorrow"));

    let Some(job) = test_parse_job_status_command("/job abc123 json") else {
        panic!("expected /job json parse");
    };
    assert_eq!(job.job_id, "abc123");
    assert_eq!(job.format, OutputFormat::Json);
}

#[test]
fn test_support_maps_resume_feedback_and_partition_modes() {
    assert_eq!(
        test_parse_resume_context_command("/resume drop"),
        Some(ResumeContextCommand::Drop)
    );

    let Some(feedback) = test_parse_session_feedback_command("/feedback up") else {
        panic!("expected /feedback up parse");
    };
    assert_eq!(feedback.direction, SessionFeedbackDirection::Up);
    assert_eq!(feedback.format, OutputFormat::Dashboard);

    let Some(partition) = test_parse_session_partition_command("/session partition chat_user json")
    else {
        panic!("expected /session partition chat_user json parse");
    };
    assert_eq!(partition.mode, Some(SessionPartitionMode::ChatUser));
    assert_eq!(partition.format, OutputFormat::Json);
    assert_eq!(
        SessionPartitionMode::ChatThreadUser.as_str(),
        "chat_thread_user"
    );
    let Some(scope_alias) = test_parse_session_partition_command("/session scope on") else {
        panic!("expected /session scope on parse");
    };
    assert_eq!(scope_alias.mode, Some(SessionPartitionMode::Chat));
    assert_eq!(scope_alias.format, OutputFormat::Dashboard);

    let Some(injection) = test_parse_session_injection_command("/session inject status json")
    else {
        panic!("expected /session inject status json parse");
    };
    assert_eq!(injection.action, SessionInjectionAction::Status);
    assert_eq!(injection.format, OutputFormat::Json);

    let Some(admin) = test_parse_session_admin_command("/session admin add 1001,1002") else {
        panic!("expected admin parse");
    };
    assert_eq!(
        admin.action,
        SessionAdminAction::Add(vec!["1001".to_string(), "1002".to_string()])
    );

    let Some(mention) = test_parse_session_mention_command("/session mention inherit json") else {
        panic!("expected /session mention inherit json parse");
    };
    assert_eq!(mention.mode, Some(SessionMentionMode::Inherit));
    assert_eq!(mention.format, OutputFormat::Json);
}

#[test]
fn test_support_managed_detectors_remain_stable() {
    assert_eq!(
        detect_managed_slash_command("/jobs"),
        Some(ManagedSlashCommand::JobsSummary)
    );
    assert_eq!(
        detect_managed_control_command("/reset"),
        Some(ManagedControlCommand::Reset)
    );
    assert_eq!(
        detect_managed_control_command("/session admin add 1001"),
        Some(ManagedControlCommand::SessionAdmin)
    );
    assert_eq!(
        detect_managed_control_command("/session mention off"),
        Some(ManagedControlCommand::SessionMention)
    );
    assert_eq!(
        detect_managed_control_command("/session inject status json"),
        Some(ManagedControlCommand::SessionInjection)
    );
}
