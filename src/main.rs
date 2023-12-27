use clap::Parser;

use hyprland::data::*;
use hyprland::prelude::*;
use hyprland::shared::*;
use hyprland::event_listener::AsyncEventListener;

use itertools::Itertools;

use tokio::{io::AsyncWriteExt, process::Command};
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use futures::executor::block_on;

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
    let workspace_id = Client::get_active().unwrap().unwrap().workspace.id;

    Clients::get().unwrap().filter(|x| x.workspace.id == workspace_id).collect()
}

async fn ask_slurp_area(workspace_clients: &Vec<hyprland::data::Client>) -> String {
    let slurp_location: &str;
    match option_env!("SLURP_LOCATION") {
        Some(x) => {slurp_location = x},
        None => {slurp_location = "slurp"}
    }

    //let args = Args::parse();

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
        .expect("unable to read window")
        .to_owned();

    return prop;
}

fn reload_areas(reload: Arc<Mutex<bool>>) {
    let mut rel = reload.lock().unwrap();
    *rel = true;
}

#[tokio::main]
async fn main() {
    let should_reload = Arc::new(Mutex::new(false));

    let workspace_clients = get_workspace_clients();
    let area_future = ask_slurp_area(&workspace_clients);

    let mut listener = AsyncEventListener::new();

    let relclone = Arc::clone(&should_reload);

    listener.add_workspace_change_handler( move |_| {
        println!("asdf");
        let relclone = Arc::clone(&relclone);
        Box::pin(async move {
            reload_areas(relclone)
        })
    });

    let listener_future = listener.start_listener_async();

    //listener.add_workspace_change_handler(a);
    //listener.add_active_monitor_change_handler( async_closure! { |_| reload_areas(reload) });
    //listener.add_window_open_handler( async_closure! { |_| reload_areas(reload) });
    //listener.add_window_close_handler( async_closure! { |_| reload_areas(reload) });
    //listener.add_window_moved_handler( async_closure! { |_| reload_areas(reload) });
    //
    
    #[allow(unused_variables)]
    let prop = get_prop(&workspace_clients, block_on(area));
    //println!("{:#?}", prop);

    println!("{:#?}", should_reload.clone().lock().unwrap());
}
