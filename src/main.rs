use std::env;

use dotenv::dotenv;

fn main() {
    dotenv().ok();

    let league_folder = env::var("LEAGUE_FOLDER").unwrap();

    println!("{}", league_folder);
}
