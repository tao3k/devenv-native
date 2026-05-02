use super::{ConstructCliCommand, must_ok, must_some, parse_construct_command, to_args};

#[test]
fn parse_construct_index_command() {
    assert_eq!(
        must_some(
            must_ok(
                parse_construct_command(&to_args(&["qianji", "construct", "index"])),
                "construct index parse should succeed",
            ),
            "construct command should be detected",
        ),
        ConstructCliCommand::Index { json: false },
    );
}

#[test]
fn parse_construct_show_command() {
    assert_eq!(
        must_some(
            must_ok(
                parse_construct_command(&to_args(&[
                    "qianji",
                    "construct",
                    "show",
                    "gateway.exclusive.bounded",
                ])),
                "construct show parse should succeed",
            ),
            "construct command should be detected",
        ),
        ConstructCliCommand::Show {
            id: "gateway.exclusive.bounded".to_string(),
            json: false,
        },
    );
}

#[test]
fn parse_construct_json_commands() {
    assert_eq!(
        must_some(
            must_ok(
                parse_construct_command(&to_args(&["qianji", "construct", "index", "--json"])),
                "construct index json parse should succeed",
            ),
            "construct command should be detected",
        ),
        ConstructCliCommand::Index { json: true },
    );
    assert_eq!(
        must_some(
            must_ok(
                parse_construct_command(&to_args(&[
                    "qianji",
                    "construct",
                    "show",
                    "gateway.exclusive.bounded",
                    "--json",
                ])),
                "construct show json parse should succeed",
            ),
            "construct command should be detected",
        ),
        ConstructCliCommand::Show {
            id: "gateway.exclusive.bounded".to_string(),
            json: true,
        },
    );
}
