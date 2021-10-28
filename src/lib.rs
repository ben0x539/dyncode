extern crate proc_macro;
use proc_macro::{TokenStream};
use proc_macro2::{Span, Ident};
use syn::{parse_macro_input, Token, Result as SynResult, LitStr, Error};
use syn::parse::{Parse, ParseStream};

use color_eyre::eyre::{Result, Report};

struct Input {
	product_key: String,
	license_key: String,
}

impl Parse for Input {
	fn parse(input: ParseStream) -> SynResult<Self> {
		let mut product_key = None;
		let mut license_key = None;
		while !input.is_empty() {
			let ident: Ident = input.parse()?;
			let field = match &ident.to_string()[..] {
				"product_key" => &mut product_key,
				"license_key" => &mut license_key,
				_ => return Err(Error::new(ident.span(), "bad parameter")),
			};
			if field.is_some() {
				return Err(Error::new(ident.span(), "duplicate parameter"));
			}
			input.parse::<Token![=]>()?;
			let lit: LitStr = input.parse()?;
			*field = Some(lit.value());

			if input.is_empty() { break; }

			input.parse::<Token![,]>()?;
		}

		let get_err = |s| move || Error::new(Span::call_site(), s);
		let product_key = product_key
			.ok_or_else(get_err("missing product_key parameter"))?;
		let license_key = license_key
			.ok_or_else(get_err("missing license_key parameter"))?;
		Ok(Input { product_key, license_key })
	}
}

#[proc_macro]
pub fn activate_license(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input);
	match activate_license_inner(input) {
		Ok(tokens) => tokens,
		Err(e) => panic!("{}", e.to_string()),
	}
}

fn activate_license_inner(input: Input) -> Result<TokenStream> {
	let rt = tokio::runtime::Runtime::new().unwrap();
	let Input { product_key, license_key } = input;
	let result = rt.block_on(get_code(&product_key, &license_key))?;
	Ok(result.parse().
		map_err(|e: proc_macro::LexError| Report::msg(e.to_string()))?)
}

async fn get_code(product_key: &str, license_key: &str) -> Result<String> {
	let url = format!("https://gist.githubusercontent.com/ben0x539/{}/raw/{}/gistfile1.txt", product_key, license_key);
	Ok(reqwest::get(&url).await?
		.error_for_status()
		.map_err(|_| Report::msg("invalid or expired license key"))?
		.text().await?)
}

