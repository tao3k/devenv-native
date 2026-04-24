use super::count_agenda_statuses;

#[test]
fn count_agenda_statuses_tracks_open_and_done_tasks() {
    let content = "\
- [ ] Open task
- [x] Done task <!-- id: review -->
- not a task
";

    let (open, done) = count_agenda_statuses(content);
    assert_eq!((open, done), (1, 1));
}
