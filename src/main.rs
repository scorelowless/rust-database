use std::io;
use clap::Parser;
use lab6::database_key::DatabaseKey;
use lab6::error::Error;
use lab6::parser::parse;

pub mod cli_args {
    use clap::{Parser, ValueEnum};

    #[derive(Debug, ValueEnum, Clone)]
    pub(crate) enum DatabaseKey {
        String,
        Int
    }

    #[derive(Parser, Debug)]
    pub(crate) struct Args {
        #[arg(short = 'c', value_enum)]
        pub(crate) key: DatabaseKey,
    }
}

fn handle_database<K: DatabaseKey>(){
    let mut db = lab6::database::Database::<K>::create_database();
    loop {
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            println!("{}", Error::CLIError)
        }
        let input = input.trim();
        match parse::<K>(input) {
            Ok(c) => match c.execute(&mut db) {
                Err(e) => println!("{e}"),
                Ok(..) => db.history.commands.push(input.to_string())
            }
            Err(e) => println!("{e}")
        }
    }
}

fn main() {
    let args = cli_args::Args::parse();
    match args.key {
        cli_args::DatabaseKey::String => handle_database::<String>(),
        cli_args::DatabaseKey::Int => handle_database::<i64>()
    }
}