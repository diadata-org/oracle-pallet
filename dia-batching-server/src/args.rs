use structopt::StructOpt;

fn parse_currency_vec(src: &str) -> SupportedCurrencies {
	let mut vec = Vec::new();
	for s in src.split(',') {
		vec.push(s.to_string());
	}
	SupportedCurrencies(vec)
}

// We need the extra struct to be able to parse the currencies to a Vec
#[derive(Debug)]
pub struct SupportedCurrencies(pub Vec<String>);

#[derive(Debug, StructOpt)]
#[structopt(name = "dia-batching-server", about = "An server for batching requests to the Dia API")]
pub struct DiaApiArgs {
	/// Iteration duration after one batch of requests
	#[structopt(short, long, default_value = "60")]
	pub iteration_timeout_in_seconds: u64,

	/// Timeout after one request
	#[structopt(short, long, default_value = "100")]
	pub request_timeout_in_milliseconds: u64,

	/// Currencies to support
	/// Each currency needs to have the format <blockchain>:<symbol>
	/// Fiat currencies need to have the format FIAT:<from>-<to>
	#[structopt(short, long,
      parse(from_str = parse_currency_vec),
      default_value = "Polkadot:DOT,Kusama:KSM,Stellar:XLM,FIAT:USD-USD,FIAT:MXN-USD,FIAT:BRL-USD,Amplitude:AMPE"
    )]
	pub supported_currencies: SupportedCurrencies,
}
