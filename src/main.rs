use clap::Parser;

use hyprland::data::*;
use hyprland::prelude::*;
use hyprland::shared::*;
use hyprland::event_listener::EventListener;

use itertools::Itertools;

use std::process::{Command, Stdio};
use std::io::Write;


/// xprop for Hyprland
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
}

trait ToSlurpArea {
    fn to_slurp_area(&self) -> String;
}

impl ToSlurpArea for Client {
    fn to_slurp_area(&self) -> String {
        return format!("{ax},{ay} {bx}x{by}",
                       ax=self.at.0,
                       ay=self.at.1,
                       bx=self.size.0,
                       by=self.size.1);
    }
}

fn ask_slurp_prop() -> Client {
    let slurp_location: &str;
    match option_env!("SLURP_LOCATION") {
        Some(x) => {slurp_location = x},
        None => {slurp_location = "slurp"}
    }

    //let args = Args::parse();
    let workspace_id = Client::get_active().unwrap().unwrap().workspace.id;

    let workspace_clients: &Vec<Client> = &(Clients::get()
                                            .unwrap()
                                            .filter(|x| x.workspace.id == workspace_id)
                                            .collect()
                                            );

    let client_sizes_slurp: String = workspace_clients
        .into_iter()
        .map(|x| x.to_slurp_area())
        .join("\n");

    let mut slurpcommand = Command::new(slurp_location)
        .arg("-r")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap_or_else(|_| panic!("failed to execute process `{slurp_location}`"));

    let mut stdin = slurpcommand.stdin.take().expect("Failed to create stdin");
    stdin.write_all(client_sizes_slurp.as_bytes())
        .expect("Failed to write to stdin");
    drop(stdin);

    let output = slurpcommand.wait_with_output().expect("Wait failed?");
    let selected_slurp_area = String::from_utf8_lossy(&output.stdout).to_string();
    let selected_slurp_area = selected_slurp_area.trim();

    let prop = workspace_clients
        .into_iter()
        .filter(|x| x.to_slurp_area() == selected_slurp_area)
        .nth(0)
        .expect("unable to read window")
        .to_owned();

    return prop;
}

fn reload_areas() {
    // reloads areas and workspace_clients
}

fn main() {
    let mut listener = EventListener::new();
    listener.add_workspace_change_handler(|_| reload_areas());
    listener.add_active_monitor_change_handler(|_| reload_areas());
    listener.add_window_open_handler(|_| reload_areas());
    listener.add_window_close_handler(|_| reload_areas());
    listener.add_window_moved_handler(|_| reload_areas());

    let prop = ask_slurp_prop();
    println!("{:#?}", prop);
}
