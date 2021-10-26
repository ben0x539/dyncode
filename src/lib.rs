extern crate proc_macro;
use proc_macro::TokenStream;

use color_eyre::eyre::Result;

const URL: &'static str = "https://gist.githubusercontent.com/ben0x539/ad3b38a8a88312dd6e8ba59d216d359d/raw/28cb00293e1fc170414b8c586b2b72fe8424dbfd/gistfile1.txt";

#[proc_macro]
pub fn toy(_item: TokenStream) -> TokenStream {
	let rt = tokio::runtime::Runtime::new().unwrap();
	let result = rt.block_on(get_code());
	result.unwrap().parse().unwrap()
}

async fn get_code() -> Result<String> {
	Ok(reqwest::get(URL).await?.text().await?)
}

