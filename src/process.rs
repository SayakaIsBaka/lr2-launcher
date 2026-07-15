use std::{path::PathBuf, process::{Command, Stdio}};

pub fn launch_game(lr2_path: &PathBuf, no_save: bool) {
    let mut command = Command::new(lr2_path);
    command.stdout(Stdio::null()).stderr(Stdio::null());

    if no_save {
        command.args(&["-ns"]);
    }

    command.spawn().unwrap();
}