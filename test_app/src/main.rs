#[tokio::main]
async fn main() {
    static BASE_URL: &str = "https://trzebnica.aqi.eco/pl";

    println!("{:?}", fetch_data::get_weekly_stats(BASE_URL).await);
}
