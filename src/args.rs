macro_rules! define_args {
	($(
		$field:ident : $short:literal, $long:literal, $desc:literal
	);* $(;)?) => {
		pub struct Args {
			$(pub $field: bool,)*
			pub files: Vec<String>,
		}

		impl Args {
			pub fn parse(args: Vec<String>) -> Self {
                let (flags, files): (Vec<_>, Vec<_>) = args.into_iter().skip(1).partition(|a| a.starts_with('-'));
				let mut result = Args {
					$($field: false,)*
					files,
				};

				for raw in flags {
					let name = raw.trim_start_matches('-');
					$(if name == $short || name == $long {
						result.$field = true;
					})*
				}

				result
			}

			pub fn get_help(&self) -> String {
				use ::std::fmt::Write;
                let mut sb = String::from(concat!(
                    env!("CARGO_PKG_NAME"),
                    " v",
                    env!("CARGO_PKG_VERSION"),
                    "\nUsage: ",
                    env!("CARGO_PKG_NAME"),
                    " [options] [path]...\nOptions:\n",
                ));
				$(writeln!(sb, "\t{:<15}\t{}", concat!("-", $short, ", --", $long), $desc).unwrap();)*
				sb
			}
		}
	};
}

define_args! {
	help:   "h", "help", "Show help";
	r#move: "m", "move", "Move files instead of copy";
}
