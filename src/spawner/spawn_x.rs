use std::{os::fd::{AsFd, AsRawFd}, process::{Command, Stdio}};




fn spawn_child(){

    let child = Command::new("sample-play/testbin/playi")
        .arg("Welp")
        .stdout(Stdio::piped())
        .spawn()
        .expect("Fail to spawn child");

    

}   