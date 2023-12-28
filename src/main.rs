use clap::Parser;

use hyprland::data::*;
use hyprland::prelude::*;
use hyprland::shared::*;
use hyprland::event_listener::EventListener;

use itertools::Itertools;

use tokio::{io::AsyncWriteExt, process::Command};
use std::process::Stdio;
use std::sync::{Arc, Mutex};

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

fn get_workspace_clients() -> Vec<hyprland::data::Client>{
    // workspace may be empty, making this unwrap fail
    let workspace_id = Client::get_active()
        .expect("Cannot get active client")  // get active expect
        .expect("No windows in active workspace")
        .workspace.id;

    return Clients::get()
        .expect("Cannot get clients")
        .filter(|x| x.workspace.id == workspace_id)
        .collect();
}

async fn ask_slurp_area(workspace_clients: &Vec<hyprland::data::Client>) -> String {
    let slurp_location: &str;
    match option_env!("SLURP_LOCATION") {
        Some(x) => {slurp_location = x},
        None => {slurp_location = "slurp"}
    }

    Args::parse();

    let client_sizes_slurp: String = workspace_clients
        .into_iter()
        .map(|x| x.to_slurp_area())
        .join("\n");

    let mut slurpcommand = Command::new(slurp_location)
        .arg("-r")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .unwrap_or_else(|_| panic!("failed to execute process `{slurp_location}`"));

    let mut stdin = slurpcommand.stdin.take().expect("Failed to create stdin");
    stdin.write_all(client_sizes_slurp.as_bytes())
        .await
        .expect("Failed to write to stdin");
    drop(stdin);

    let output = slurpcommand
        .wait_with_output()
        .await
        .expect("Wait failed?");

    return String::from_utf8_lossy(&output.stdout).to_string().trim().to_string();
}

fn get_prop(workspace_clients: &Vec<Client>, selected_slurp_area: String) -> Client {

    let prop = workspace_clients
        .into_iter()
        .filter(|x| x.to_slurp_area() == selected_slurp_area)
        .nth(0)
        .expect("Cannot find client with selected area (selection may be cancelled)")
        .to_owned();

    return prop;
}

fn reload_areas(reload: &Arc<Mutex<bool>>) {
    let mut rel = reload.lock().expect("Cannot unlock");
    *rel = true;
}

#[tokio::main]
async fn main() {
    let should_reload = Arc::new(Mutex::new(false));

    let mut listener = EventListener::new();

    let reload = should_reload.clone();
    listener.add_workspace_change_handler( move |_| { reload_areas(&reload)} );
    let reload = should_reload.clone();
    listener.add_active_monitor_change_handler( move |_| reload_areas(&reload) );
    let reload = should_reload.clone();
    listener.add_window_open_handler( move |_| reload_areas(&reload) );
    let reload = should_reload.clone();
    listener.add_window_close_handler( move |_| reload_areas(&reload) );
    let reload = should_reload.clone();
    listener.add_window_moved_handler( move |_| reload_areas(&reload) );

    let listener_future = tokio::spawn(
            async move {
                listener
                    .start_listener_async()
                    .await
                    .expect("Cannot spawn listener");
            }
        );  // this starts listening asynchronously


    let mut workspace_clients = get_workspace_clients();
    let cloned_workspace_clients = workspace_clients.clone();
    let mut area_future = ask_slurp_area(&cloned_workspace_clients);

    loop {
        //println!("newloop");
        tokio::select! {
            area_str = area_future => {
                let prop = get_prop(&workspace_clients, area_str);
                println!("{:#?}", prop);
                break;
            }

            _ = async {
                // this is fast and dirty. we should use non-loop-checking tools instead
                loop {
                    let mut rel = should_reload
                        .lock()
                        .expect("Cannot unlock should_reload");
                    if *rel {
                        // if should_reload, lock and return this future to reload workspaces
                        *rel = false;
                        break;
                    }
                    drop(rel);  // this is stupid.. but only this way we unlock reload
                    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                }
                return;
            } => {
                workspace_clients = get_workspace_clients();
                area_future = ask_slurp_area(&workspace_clients);
            }

        }
    }
    //println!("{:#?}",should_reload.clone().lock().unwrap());

    listener_future.abort();  // this is useless
}
