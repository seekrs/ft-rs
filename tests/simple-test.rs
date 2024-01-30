use ft_rs::FtClient;

#[tokio::test(flavor = "multi_thread")]
async fn main() -> ft_rs::Result<()> {
  let client = FtClient::from_app(
    std::env::var("FT_RS_TEST_UID").expect("FT_RS_TEST_UID not set"), 
    std::env::var("FT_RS_TEST_SECRET").expect("FT_RS_TEST_SECRET not set")
  )?;
  let token = client.fetch_token().await?;
  println!("Token: {:?}", token);
  Ok(())
}