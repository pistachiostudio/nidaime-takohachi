use nidaime_takohachi::utils;

#[tokio::main]
async fn main() {
    println!("Testing get_what_today function...\n");

    // Test with today's date
    let result = utils::get_what_today(10, 8).await;
    println!("10月8日: {}\n", result);

    // Test with another date
    let result = utils::get_what_today(1, 1).await;
    println!("1月1日: {}\n", result);

    // Test with another date
    let result = utils::get_what_today(12, 25).await;
    println!("12月25日: {}\n", result);
}
