use structopt::StructOpt;

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
	#[structopt(short, long, default_value = "Vec::default()")]
	pub supported_currencies: Option<Vec<String>>,
}
