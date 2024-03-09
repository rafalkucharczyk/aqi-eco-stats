use fetch_data;

#[tokio::main]
async fn main() {
    println!("{:?}", fetch_data::get_data().await);
}
