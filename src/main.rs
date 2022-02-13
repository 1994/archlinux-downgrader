use chrono::Utc;
use dialoguer::{Confirm, Input};
use sudo::RunningAs;

use crate::pacman_helper::{Pkg, PkgType};

mod pacman_helper;

fn main() {
    let r = sudo::check();
    if r != RunningAs::Root {
        println!("you need run with root");
        return;
    }

    let t = Utc::today().format("%Y-%m-%d").to_string();
    let input: String = Input::new()
        .with_prompt("输入想回滚的日期?")
        // .with_initial_text("Yes")
        .default(t)
        .interact_text()
        .expect("input error");

    let value = input.as_str();
    let filter = pacman_helper::Filter::new(value);
    println!("{:?}", filter);

    let result = pacman_helper::execute(&filter);

    let installed: Vec<&Pkg> = result
        .iter()
        .filter(|x| x.p_type.eq(&PkgType::Installed))
        .collect();

    let upgraded: Vec<&Pkg> = result
        .iter()
        .filter(|x| x.p_type.eq(&PkgType::Upgraded))
        .collect();

    println!("{:?}", &upgraded);

    if Confirm::new()
        .with_prompt("是否要全部降级?")
        .interact()
        .unwrap()
    {
        pacman_helper::do_downgrade(upgraded);
    }

    println!("{:?}", &installed);
    if Confirm::new()
        .with_prompt("是否要全部卸载?")
        .interact()
        .unwrap()
    {
        pacman_helper::do_uninstalled(installed);
    }
}
