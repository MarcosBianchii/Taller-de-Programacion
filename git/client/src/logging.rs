use crate::plumbing::commands::get_userconfig;
use std::{
    fs::OpenOptions,
    io::{self, Write},
};

pub enum LogMsgStatus {
    ErrOnExecution(String),
    CorrectExecution,
}

#[allow(dead_code)]
fn register_log(log_msg: String) -> io::Result<()> {
    let usr_config = get_userconfig()?;
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(usr_config.get_log_path())?;

    file.write_all(log_msg.as_bytes())?;

    Ok(())
}

pub fn log_command(command: String, execution_status: LogMsgStatus) -> io::Result<()> {
    let usr_config = get_userconfig()?;
    let log_mode = usr_config.get_log_mode();
    let time = chrono::Local::now().format("%d-%m-%Y %H:%M:%S");

    let log_msg = match execution_status {
        LogMsgStatus::CorrectExecution => {
            if log_mode == "errors" {
                return Ok(());
            };
            format!(
                "{time}    {command}    {}\n",
                String::from("Command executed as expected")
            )
        }
        LogMsgStatus::ErrOnExecution(err_msg) => {
            format!("[Error] {time}    {command}    {err_msg}\n")
        }
    };

    register_log(log_msg)?;
    Ok(())
}
