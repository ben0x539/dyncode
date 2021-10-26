extern crate proc_macro;
use proc_macro::{TokenTree, TokenStream};
use proc_macro::TokenTree::*;

use color_eyre::eyre::{bail, Result, Report};

#[proc_macro]
pub fn activate_license(input: TokenStream) -> TokenStream {
	match activate_license_inner(input) {
		Ok(tokens) => tokens,
		Err(e) => panic!("{}", e.to_string()),
	}
}

fn activate_license_inner(input: TokenStream) -> Result<TokenStream> {
	let rt = tokio::runtime::Runtime::new().unwrap();
	let (product_key, license_key) = parse_keys(input)?;
	let result = rt.block_on(get_code(&product_key, &license_key))?;
	Ok(result.parse().
		map_err(|e: proc_macro::LexError| Report::msg(e.to_string()))?)
}

fn get_ident(input: &mut impl Iterator<Item=TokenTree>) -> Result<String> {
	match input.next() {
		Some(Ident(ident)) => Ok(ident.to_string()),
		_ => bail!("bad ident"),
	}
}

fn get_str_lit(input: &mut impl Iterator<Item=TokenTree>) -> Result<String> {
	let s = match input.next() {
		Some(Literal(literal)) => literal.to_string(),
		_ => bail!("bad literal"),
	};
	if !s.starts_with("\"") {
		bail!("bad literal");
	}
	Ok(s[1..s.len()-1].to_string())
}

fn want_punct(input: &mut impl Iterator<Item=TokenTree>, c: char)
		-> Result<()> {
	match input.next() {
		Some(Punct(punct)) if punct.as_char() == c => Ok(()),
		_ => bail!("bad punct"),
	}
}

fn parse_keys(tokens: TokenStream) -> Result<(String, String)> {
	let mut tokens = tokens.into_iter();

	let first_ident = get_ident(&mut tokens)?;
	want_punct(&mut tokens, '=');
	let first_str_lit = get_str_lit(&mut tokens)?;
	want_punct(&mut tokens, ',');
	let second_ident = get_ident(&mut tokens)?;
	want_punct(&mut tokens, '=');
	let second_str_lit = get_str_lit(&mut tokens)?;

	match (&first_ident.to_string()[..], &second_ident.to_string()[..]) {
		("product_key", "license_key") => Ok((first_str_lit, second_str_lit)),
		("license_key", "product_key") => Ok((second_str_lit, first_str_lit)),
		_ => bail!("bad input 9"),
	}
}

async fn get_code(product_key: &str, license_key: &str) -> Result<String> {
	let url = format!("https://gist.githubusercontent.com/ben0x539/{}/raw/{}/gistfile1.txt", product_key, license_key);
	Ok(reqwest::get(&url).await?
		.error_for_status()?
		.text().await?)
}

