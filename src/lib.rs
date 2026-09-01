pub mod error {
    #[derive(thiserror::Error, Debug)]
    #[derive(PartialEq)]
    pub enum Error {
        #[error("Record cannot be inserted into the given table - length mismatch")]
        RecordInvalidLength,
        #[error("Record cannot be inserted into the given table - duplicate key")]
        RecordInvalidKey,
        #[error("Record cannot be inserted into the given table - field name mismatch")]
        RecordInvalidFields,
        #[error("Record cannot be inserted into the given table - field type mismatch")]
        RecordInvalidFieldType,
        #[error("The record does not have this field")]
        RecordNoSuchField,
        #[error("Table's key is not in table's fields")]
        TableInvalidKey,
        #[error("Table's key is not of valid type")]
        TableInvalidKeyType,
        #[error("Record does not have a unique key in the table")]
        TableDuplicateKey,
        #[error("Error in program code, developer's fault")]
        BadCode,
        #[error("Record cannot be removed from the table, as the table does not contain it")]
        TableNonExistentKey,
        #[error("Table with this name already present in the database")]
        DuplicateTable,
        #[error("Table with this name does not exist in the database")]
        NonexistentTable,
        #[error("The selection does not have this field")]
        SelectNoSuchField,
        #[error("Could not parse the command: {0}")]
        CommandParseError(String),
        #[error("Problem occurred with file")]
        IOError,
        #[error("Problem occurred with console")]
        CLIError,
    }
}

pub mod database_key {
    use crate::value::{Value, ValueType};

    pub trait DatabaseKey: Ord {
        fn from_value(v: &Value) -> Option<Self> where Self: Sized;
        fn is_value_type(v: &ValueType) -> bool;
        fn from_str(s: &str) -> Option<Self> where Self: Sized;
    }
    impl DatabaseKey for i64 {
        fn from_value(v: &Value) -> Option<Self> {
            if let Value::Int(n) = v {
                Some(*n)
            } else {
                None
            }
        }

        fn is_value_type(v: &ValueType) -> bool {
            matches!(v, ValueType::Int)
        }

        fn from_str(s: &str) -> Option<Self> {
            <i64 as std::str::FromStr>::from_str(s).ok()
        }
    }
    impl DatabaseKey for String {
        fn from_value(v: &Value) -> Option<Self> {
            if let Value::String(s) = v {
                Some(s.clone())
            } else {
                None
            }
        }

        fn is_value_type(v: &ValueType) -> bool {
            matches!(v, ValueType::String)
        }

        fn from_str(s: &str) -> Option<Self> {
            Some(s.to_string())
        }
    }
}

pub mod condition {
    use crate::error::Error;
    use crate::record::Record;
    use crate::value::Value;

    enum Op {
        E,
        NE,
        L,
        LE,
        G,
        GE,
    }
    pub struct Condition {
        field: String,
        operation: Op,
        value: Value,
    }

    impl Condition {
        pub fn equal(field: String, value: Value) -> Condition {
            Condition {
                field,
                operation: Op::E,
                value,
            }
        }
        pub fn not_equal(field: String, value: Value) -> Condition {
            Condition {
                field,
                operation: Op::NE,
                value,
            }
        }
        pub fn less_than(field: String, value: Value) -> Condition {
            Condition {
                field,
                operation: Op::L,
                value,
            }
        }
        pub fn less_than_or_equal(field: String, value: Value) -> Condition {
            Condition {
                field,
                operation: Op::LE,
                value,
            }
        }
        pub fn greater_than(field: String, value: Value) -> Condition {
            Condition {
                field,
                operation: Op::G,
                value,
            }
        }
        pub fn greater_than_or_equal(field: String, value: Value) -> Condition {
            Condition {
                field,
                operation: Op::GE,
                value,
            }
        }
    }

    impl Record {
        pub fn check_condition(&self, cond: &Condition) -> Result<bool, Error> {
            let v = match self.values.get(&cond.field) {
                None => return Err(Error::RecordNoSuchField),
                Some(x) => x
            };
            match cond.operation {
                Op::E => Ok(*v == cond.value),
                Op::NE => Ok(*v != cond.value),
                Op::L => Ok(*v < cond.value),
                Op::LE => Ok(*v <= cond.value),
                Op::G => Ok(*v > cond.value),
                Op::GE => Ok(*v >= cond.value)
            }
        }
    }
}
pub mod value {
    use std::fmt::Formatter;

    pub enum Value {
        Bool(bool),
        String(String),
        Int(i64),
        Float(f64),
    }

    pub enum ValueType {
        Bool,
        String,
        Int,
        Float
    }

    impl PartialOrd for Value {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            match (self, other) {
                (Value::Bool(a), Value::Bool(b)) => a.partial_cmp(b),
                (Value::String(a), Value::String(b)) => a.partial_cmp(b),
                (Value::Int(a), Value::Int(b)) => a.partial_cmp(b),
                (Value::Float(a), Value::Float(b)) => a.partial_cmp(b),
                (Value::Int(a), Value::Float(b)) => (*a as f64).partial_cmp(b),
                (Value::Float(a), Value::Int(b)) => a.partial_cmp(&(*b as f64)),
                _ => None
            }
        }
    }

    impl PartialEq for Value {
        fn eq(&self, other: &Self) -> bool {
            match (self, other) {
                (Value::Bool(a), Value::Bool(b)) => a == b,
                (Value::String(a), Value::String(b)) => a == b,
                (Value::Int(a), Value::Int(b)) => a == b,
                (Value::Float(a), Value::Float(b)) => a == b,
                (Value::Int(a), Value::Float(b)) => (*a as f64) == *b,
                (Value::Float(a), Value::Int(b)) => *a == (*b as f64),
                _ => false,
            }
        }
    }

    impl std::fmt::Display for Value {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            match self {
                Value::Bool(b) => write!(f, "{b}"),
                Value::Int(n) => write!(f, "{n}"),
                Value::Float(f64) => write!(f, "{:6.2}", f64),
                Value::String(s) => write!(f, "\"{s}\""),
            }
        }
    }

    impl Value {
        pub(crate) fn assignable_to(&self, other: &ValueType) -> bool {
            std::mem::discriminant(&self.to_type()) == std::mem::discriminant(other)
        }

        fn to_type(&self) -> ValueType {
            match self {
                Value::Bool(_) => ValueType::Bool,
                Value::String(_) => ValueType::String,
                Value::Int(_) => ValueType::Int,
                Value::Float(_) => ValueType::Float
            }
        }
    }
}

mod record {
    use crate::error::Error;
    use crate::value::Value;
    use std::collections::HashMap;

    pub struct Record {
        pub(crate) values: HashMap<String, Value>
    }

    impl Record {
        pub fn new(values: HashMap<String, Value>) -> Record {
            Record {
                values
            }
        }

        pub fn get_fields(&self, fields: &Vec<String>) -> Result<Vec<&Value>, Error> {
            let mut result = Vec::new();
            for field in fields {
                if !self.values.contains_key(field) {
                    return Err(Error::RecordInvalidFields);
                }
                let value = match self.values.get(field) {
                    None => return Err(Error::BadCode),
                    Some(a) => a
                };
                result.push(value);
            }
            Ok(result)
        }
    }
}

mod table {
    use crate::condition::Condition;
    use crate::database_key::DatabaseKey;
    use crate::error::Error;
    use crate::record::Record;
    use std::collections::btree_map::Values;
    use std::collections::{BTreeMap, HashMap};

    pub struct Table<K: DatabaseKey> {
        key: String, // name of the column containing the key
        fields: HashMap<String, crate::value::ValueType>, // Value contains the info about field data type
        records: BTreeMap<K, Record>
    }

    impl<K: DatabaseKey> Table<K> {
        pub(crate) fn create_table(key: String, fields: HashMap<String, crate::value::ValueType>) -> Result<Table<K>, Error> {
            if !fields.contains_key(&key) {
                Err(Error::TableInvalidKey)
            } else if !K::is_value_type(&fields[&key]) {
                Err(Error::TableInvalidKeyType)
            } else {
                Ok(Table {
                    key,
                    fields,
                    records: BTreeMap::new()
                })
            }
        }

        fn validate_record(&self, record: &Record) -> Result<(), Error> {
            // checks if the record could be theoretically inserted into the table
            // i.e. if it has correct length, field names and field types
            // doesn't check for duplicate keys
            if self.fields.len() != record.values.len() {
                return Err(Error::RecordInvalidLength)
            }
            for field in &self.fields {
                let record_field = match record.values.get(field.0) {
                    None => return Err(Error::RecordInvalidFields),
                    Some(a) => a
                };
                if !record_field.assignable_to(field.1) {
                    return Err(Error::RecordInvalidFieldType)
                }
            }
            Ok(())
        }

        pub fn remove_record(&mut self, key: K) -> Result<(), Error> {
            match self.records.remove(&key){
                None => Err(Error::TableNonExistentKey),
                Some(_) => Ok(())
            }
        }

        pub fn record_values(&self) -> Values<'_, K, Record> {
            self.records.values()
        }

        pub fn record_values_where(&self, cond: &Condition) -> Result<Vec<&Record>, Error> {
            let mut vec = Vec::new();
            for record in self.records.values() {
                let r = record.check_condition(cond)?;
                if r {
                    vec.push(record);
                }
            }
            Ok(vec)
        }

        pub fn insert_record(&mut self, record: Record) -> Result<(), Error> {
            self.validate_record(&record)?;
            let record_key = match record.values.get(&self.key) {
                None => return Err(Error::BadCode),
                Some(a) => a
            };
            let record_key = K::from_value(record_key).ok_or(Error::BadCode)?;
            if self.records.contains_key(&record_key) {
                return Err(Error::TableDuplicateKey)
            }
            match self.records.insert(record_key, record) {
                None => Ok(()),
                Some(_) => Err(Error::BadCode)
            }
        }
    }
}

pub mod select {
    use crate::value::Value;

    pub struct SelectResult<'a> {
        pub(crate) fields: Vec<String>, // names of the fields
        pub(crate) values: Vec<Vec<&'a Value>>
    }

    impl<'a> SelectResult<'a> {
        pub(crate) fn new(fields: Vec<String>) -> SelectResult<'a> {
            SelectResult {
                fields,
                values: Vec::new()
            }
        }

        pub(crate) fn add(&mut self, record: Vec<&'a Value>) {
            self.values.push(record);
        }
    }


    impl std::fmt::Display for SelectResult<'_> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            for str in &self.fields {
                write!(f, "{str} ")?;
            }
            for record in &self.values {
                writeln!(f)?;
                for val in record {
                    write!(f, "{val} ")?;
                }
            }
            Ok(())
        }
    }
}

pub mod database {
    use std::cmp::Ordering;
    use crate::condition::Condition;
    use crate::database_key::DatabaseKey;
    use crate::error::Error;
    use crate::record::Record;
    use crate::select::SelectResult;
    use crate::table::Table;
    use crate::value::Value;
    use std::collections::HashMap;
    use crate::database::history::History;

    pub mod history {
        pub struct History {
            pub commands: Vec<String>
        }

        impl History {
            pub(crate) fn new() -> History {
                History {
                    commands: Vec::new()
                }
            }
        }
    }

    pub struct Database<K: DatabaseKey> {
        tables: HashMap<String, Table<K>>,
        pub history: History
    }

    impl<K: DatabaseKey> Database<K> {
        pub fn create_database() -> Database<K> {
            Database {
                tables: HashMap::new(),
                history: History::new()
            }
        }

        fn get_mut_table(&mut self, name: String) -> Result<&mut Table<K>, Error> {
            match self.tables.get_mut(&name) {
                None => Err(Error::NonexistentTable),
                Some(t) => Ok(t)
            }
        }

        fn get_table(&self, name: String) -> Result<&Table<K>, Error> {
            match self.tables.get(&name) {
                None => Err(Error::NonexistentTable),
                Some(t) => Ok(t)
            }
        }

        pub fn create_table(&mut self, name: String, key: String, fields: HashMap<String, crate::value::ValueType>) -> Result<(), Error> {
            if self.tables.contains_key(&name) {
                return Err(Error::DuplicateTable)
            }
            let table = Table::create_table(key, fields)?;
            match self.tables.insert(name, table){
                None => Ok(()),
                Some(_) => Err(Error::BadCode)
            }
        }

        pub fn delete_key(&mut self, table: String, key: K) -> Result<(), Error> {
            let table = self.get_mut_table(table)?;
            table.remove_record(key)?;
            Ok(())
        }

        pub fn select_from_table(&'_ mut self, table_name: String, fields: Vec<String>) -> Result<SelectResult<'_>, Error> {
            let mut result = SelectResult::new(fields.clone());
            let table = self.get_table(table_name)?;
            for record in table.record_values() {
                result.add(record.get_fields(&fields)?);
            }
            Ok(result)
        }

        pub fn select_from_table_where(&'_ mut self, table_name: String, fields: Vec<String>, cond: Condition) -> Result<SelectResult<'_>, Error> {
            let mut result = SelectResult::new(fields.clone());
            let table = self.get_table(table_name)?;
            for record in table.record_values_where(&cond)? {
                result.add(record.get_fields(&fields)?);
            }
            Ok(result)
        }

        pub fn order_by(mut select_result: SelectResult, field: String) -> Result<SelectResult, Error> {
            let ind = match select_result.fields.iter().position(|x| *x == field) {
                None => return Err(Error::SelectNoSuchField),
                Some(i) => i
            };
            select_result.values.sort_by(|x, y| {
                x[ind].partial_cmp(y[ind]).unwrap_or(Ordering::Equal)
                // the "or" situation should never happen since x[ind] and y[ind] should have
                // the same underlying data type, which guarantees comparability
            });
            Ok(select_result)
        }

        pub fn limit(mut select_result: SelectResult, number: usize) -> SelectResult {
            if number < select_result.values.len() {
                select_result.values = select_result.values[0..number].to_owned();
            }
            select_result
        }

        pub fn insert_into_table(&mut self, record_values: HashMap<String, Value>, table_name: String) -> Result<(), Error> {
            let table = self.get_mut_table(table_name)?;
            let record = Record::new(record_values);
            table.insert_record(record)
        }
    }

    #[cfg(test)]
    mod db_tests {
        use std::collections::HashMap;
        use crate::condition::Condition;
        use crate::database::Database;
        use crate::value::{Value, ValueType};

        #[test]
        fn test_create_database_int() {
            let _db = Database::<i64>::create_database();
        }

        #[test]
        fn test_create_table(){
            let mut db = Database::<String>::create_database();
            let mut fields = HashMap::new();
            fields.insert("key".to_string(), ValueType::String);
            fields.insert("a".to_string(), ValueType::Int);
            db.create_table("table".to_string(), "key".to_string(), fields).expect("Errored on creating table");
        }

        #[test]
        fn test_insert_record(){
            let mut db = Database::<String>::create_database();
            let mut fields = HashMap::new();
            fields.insert("key".to_string(), ValueType::String);
            fields.insert("a".to_string(), ValueType::Int);
            db.create_table("table".to_string(), "key".to_string(), fields).expect("Errored on creating table");
            let mut values = HashMap::new();
            values.insert("key".to_string(), Value::String("wua".to_string()));
            values.insert("a".to_string(), Value::Int(69));
            db.insert_into_table(values, "table".to_string()).expect("Errored on inserting into table");
        }

        #[test]
        fn test_delete_record(){
            let mut db = Database::<String>::create_database();
            let mut fields = HashMap::new();
            fields.insert("key".to_string(), ValueType::String);
            fields.insert("a".to_string(), ValueType::Int);
            db.create_table("table".to_string(), "key".to_string(), fields).expect("Errored on creating table");
            let mut values = HashMap::new();
            values.insert("key".to_string(), Value::String("wua".to_string()));
            values.insert("a".to_string(), Value::Int(69));
            db.insert_into_table(values, "table".to_string()).expect("Errored on inserting into table");
            db.delete_key("table".to_string(), "wua".to_string()).expect("Errored on deleting from table");
        }

        #[test]
        fn test_select(){
            let mut db = Database::<String>::create_database();
            let mut fields = HashMap::new();
            fields.insert("key".to_string(), ValueType::String);
            fields.insert("a".to_string(), ValueType::Int);
            db.create_table("table".to_string(), "key".to_string(), fields).expect("Errored on creating table");
            let mut values = HashMap::new();
            values.insert("key".to_string(), Value::String("wua".to_string()));
            values.insert("a".to_string(), Value::Int(69));
            db.insert_into_table(values, "table".to_string()).expect("Errored on inserting into table");
            db.select_from_table("table".to_string(), vec!["key".to_string(), "a".to_string()]).expect("Errored on selecting");
        }

        #[test]
        fn test_select_where_limit_order(){
            let mut db = Database::<i64>::create_database();
            let mut fields = HashMap::new();
            fields.insert("key".to_string(), ValueType::Int);
            fields.insert("a".to_string(), ValueType::Float);
            db.create_table("table".to_string(), "key".to_string(), fields).expect("Errored on creating table");
            let mut values = HashMap::new();
            values.insert("key".to_string(), Value::Int(21));
            values.insert("a".to_string(), Value::Float(3.7));
            db.insert_into_table(values, "table".to_string()).expect("Errored on inserting into table");
            let cond = Condition::equal("key".to_string(), Value::Int(21));
            let result = db.select_from_table_where("table".to_string(), vec!["key".to_string(), "a".to_string()], cond).expect("Errored on selecting");
            let result = Database::<i64>::order_by(result, "a".to_string()).expect("Errored on ordering");
            let _result = Database::<i64>::limit(result, 3);
        }


    }
}

pub mod parser {
    use std::collections::HashMap;
    use std::str::FromStr;
    use crate::condition::Condition;
    use crate::database_key::DatabaseKey;
    use crate::error::Error;
    use crate::value::{Value, ValueType};
    use crate::parser::command::Command;

    mod command {
        use std::collections::HashMap;
        use crate::condition::Condition;
        use crate::database::Database;
        use crate::database_key::DatabaseKey;
        use crate::error::Error;
        use crate::value::{Value, ValueType};
        use crate::file_io::{read_from_file, save_to_file};
        use crate::parser::parse;

        pub enum Command<K: DatabaseKey> {
            Create {
                table: String,
                key: String,
                fields: HashMap<String, ValueType>,
                input: String
            },
            Insert {
                table: String,
                values: HashMap<String, Value>,
                input: String
            },
            Delete {
                key: K,
                table: String,
                input: String
            },
            SaveAs {
                filepath: String,
                input: String
            },
            ReadFrom {
                filepath: String,
                input: String
            },
            Select {
                fields: Vec<String>,
                table: String,
                condition: Option<Condition>,
                order_by: Option<String>,
                limit: Option<usize>,
                input: String
            }
        }

        impl<K: DatabaseKey> Command<K> {
            pub fn execute(self, db: &mut Database<K>) -> Result<(), Error> {
                match self {
                    Command::Create { table, key, fields, input } => Self::create_fn(db, table, key, fields, input),
                    Command::Insert { table, values, input } => Self::insert_fn(db, table, values, input),
                    Command::Delete { key, table, input } => Self::delete_fn(db, key, table, input),
                    Command::SaveAs { filepath, input } => Self::save_as_fn(db, filepath, input),
                    Command::ReadFrom { filepath, input } => Self::read_from_fn(db, filepath, input),
                    Command::Select { fields, table, condition, order_by, limit, input } => Self::select_fn(db, fields, table, condition, order_by, limit, input)
                }
            }

            fn create_fn(db: &mut Database<K>, table: String, key: String, fields: HashMap<String, ValueType>, input: String) -> Result<(), Error> {
                let res = db.create_table(table, key, fields);
                if res.is_ok() {
                    db.history.commands.push(input);
                }
                res
            }

            fn insert_fn(db: &mut Database<K>, table: String, values: HashMap<String, Value>, input: String) -> Result<(), Error> {
                let res = db.insert_into_table(values, table);
                if res.is_ok() {
                    db.history.commands.push(input);
                }
                res
            }

            fn delete_fn(db: &mut Database<K>, key: K, table: String, input: String) -> Result<(), Error> {
                let res = db.delete_key(table, key);
                if res.is_ok() {
                    db.history.commands.push(input);
                }
                res
            }

            fn save_as_fn(db: &mut Database<K>, filepath: String, input: String) -> Result<(), Error> {
                let res = save_to_file(filepath, &(db.history.commands));
                if res.is_ok() {
                    db.history.commands.push(input);
                }
                res
            }

            fn read_from_fn(db: &mut Database<K>, filepath: String, input: String) -> Result<(), Error> {
                for str in read_from_file(filepath)? {
                    parse(&str)?.execute(db)?;
                }
                db.history.commands.push(input);
                Ok(())
            }

            fn select_fn(db: &mut Database<K>, fields: Vec<String>, table: String, condition: Option<Condition>, order_by: Option<String>, limit: Option<usize>, input: String) -> Result<(), Error> {
                let mut result = match condition {
                    None => db.select_from_table(table, fields)?,
                    Some(cond) => db.select_from_table_where(table, fields, cond)?
                };
                if let Some(s) = order_by {
                    result = Database::<K>::order_by(result, s)?
                }
                if let Some(n) = limit {
                    result = Database::<K>::limit(result, n)
                }
                println!("{}", result);
                db.history.commands.push(input);
                Ok(())
            }
        }
    }

    fn read_word(input: &str) -> (&str, &str) {
        read_until(input, ' ')
    }

    fn read_until(input: &str, stop: char) -> (&str, &str) {
        let input = input.trim_start();
        if let Some(pos) = input.find(stop) {
            let (word, input) = input.split_at(pos);
            (input, word.trim())
        } else {
            ("", input)
        }
    }

    fn read_string_with_quotes(input: &str) -> (&str, &str) {
        let input = input.trim_start();
        if let Some(input_stripped) = input.strip_prefix('\"') {
            if let Some(pos) = input_stripped.find('\"') {
                let (word, input) = input.split_at(pos + 2);
                (input, word)
            } else {
                (input, "")
            }
        } else {
            read_word(input)
        }
    }
    fn read_string(input: &str) -> (&str, &str) {
        let (input, word) = read_string_with_quotes(input);
        if word.starts_with('\"') {
            (input, &word[1..word.len()-1])
        } else {
            (input, word)
        }
    }

    pub fn parse<K: DatabaseKey>(input: &str) -> Result<Command<K>, Error> {
        let input_copy = input;
        let (input, command_str) = read_word(input);
        match command_str.to_uppercase().as_str() {
            "CREATE" => parse_create(input, input_copy),
            "INSERT" => parse_insert(input, input_copy),
            "DELETE" => parse_delete(input, input_copy),
            "SELECT" => parse_select(input, input_copy),
            "SAVE_AS" => parse_save_as(input, input_copy),
            "READ_FROM" => parse_read_from(input, input_copy),
            s => Err(Error::CommandParseError(format!("{s} is not a valid command")))
        }
    }
    fn parse_create<K: DatabaseKey>(input: &str, input_copy : &str) -> Result<Command<K>, Error> {
        let (input, table) = read_string(input);
        let (input, key) = parse_key(input)?;
        let fields = parse_fields(input)?;
        Ok(Command::Create {
            table: table.to_string(),
            key: key.to_string(),
            fields,
            input: input_copy.to_string()
        })
    }

    fn parse_key(input: &str) -> Result<(&str, &str), Error> {
        let (input, command_str) = read_word(input);
        if command_str.to_uppercase().as_str() != "KEY" {
            return Err(Error::CommandParseError(format!("keyword KEY expected, got {command_str}")))
        }
        let (input, name) = read_string(input);
        if name.is_empty() {
            Err(Error::CommandParseError("missing key name".to_string()))
        } else {
            Ok((input, name))
        }
    }

    fn parse_fields(input: &str) -> Result<HashMap<String, ValueType>, Error> {
        let (input, command_str) = read_word(input);
        if command_str.to_uppercase().as_str() != "FIELDS" {
            return Err(Error::CommandParseError(format!("keyword FIELDS expected, got {command_str}")))
        }
        let mut fields = HashMap::new();
        let mut input = input;
        while !input.trim().is_empty() {
            let (inp, field, val_type) = parse_field_type(input)?;
            input = inp;
            fields.insert(field.to_string(), val_type);
        }
        Ok(fields)
    }

    fn parse_field_type(input: &str) -> Result<(&str, &str, ValueType), Error> {
        let (input, field) = read_until(input, ':');
        if field.contains(' ') {
            return Err(Error::CommandParseError("field name should not contain spaces".to_string()))
        }
        let (input, val_type) = read_until(&input[1..], ',');
        let val_type = match val_type.trim().to_uppercase().as_str() {
            "INT" => ValueType::Int,
            "STRING" => ValueType::String,
            "FLOAT" => ValueType::Float,
            "BOOL" => ValueType::Bool,
            s => return Err(Error::CommandParseError(format!("no such type as {s}")))
        };
        if input.is_empty() {
            Ok((input, field, val_type))
        } else {
            Ok((&input[1..], field, val_type)) // skipping the ',' on the beginning of input
        }
    }

    fn parse_insert<K: DatabaseKey>(input: &str, input_copy: &str) -> Result<Command<K>, Error> {
        let (input, field_values) = parse_fields_value(input)?;
        let (input, command_str) = read_word(input);
        if command_str.to_uppercase().as_str() != "INTO" {
            return Err(Error::CommandParseError(format!("keyword INTO expected, got {command_str}")))
        }
        let (input, table) = read_string(input);
        if !input.trim().is_empty() {
            Err(Error::CommandParseError(format!("excess characters at the end: {input}")))
        } else {
            Ok(Command::Insert {
                table: table.to_string(),
                values: field_values,
                input: input_copy.to_string()
            })
        }
    }

    fn parse_fields_value(input: &str) -> Result<(&str, HashMap<String, Value>), Error> {
        let mut fields = HashMap::new();
        let mut input = input;
        while input.split_whitespace().collect::<Vec<_>>().len() > 2 {
            let (inp, field, val_type) = parse_field_value(input)?;
            input = inp;
            fields.insert(field.to_string(), val_type);
        }
        Ok((input, fields))
    }

    fn parse_field_value(input: &str) -> Result<(&str, &str, Value), Error> {
        let (input, field) = read_until(input, '=');
        if field.contains(' ') {
            return Err(Error::CommandParseError("field name should not contain spaces".to_string()))
        }
        //let (input, val_type) = read_string_with_quotes(&input[1..]);
        let (input, val_type) = read_until(&input[1..], ',');

        if input.is_empty() {
            let (input, val_type) = read_string_with_quotes(val_type); // whole input is in val_type
            let value = parse_value(val_type)?;
            Ok((input, field, value))
        } else {
            let value = parse_value(val_type)?;
            Ok((&input[1..], field, value)) // skipping the ',' on the beginning of input
        }
    }

    fn parse_value(value: &str) -> Result<Value, Error> {
        if value.is_empty() {
            Err(Error::CommandParseError("failed to get value".to_string()))
        } else if value.starts_with('\"') {
            if value.ends_with('\"') && value.len() > 1 {
                Ok(Value::String(value[1..value.len() - 1].to_string()))
            } else {
                Err(Error::CommandParseError("non matching \" character".to_string()))
            }
        } else if value.contains('.') {
            Ok(Value::Float(f64::from_str(value).map_err(|_| Error::CommandParseError(format!("error when parsing float value {value}")))?))
        } else if value.to_uppercase().as_str() == "TRUE" || value.to_uppercase().as_str() == "FALSE" {
            Ok(Value::Bool(value.starts_with('T')))
        } else {
            Ok(Value::Int(<i64 as FromStr>::from_str(value).map_err(|_| Error::CommandParseError(format!("error when parsing int value {value}")))?))
        }
    }

    fn parse_delete<K: DatabaseKey>(input: &str, input_copy: &str) -> Result<Command<K>, Error> {
        let (input, key) = read_string(input);
        let key = match K::from_str(key) {
            Some(k) => k,
            None => return Err(Error::CommandParseError("invalid key type".to_string()))
        };
        let (input, command_str) = read_word(input);
        if command_str.to_uppercase().as_str() != "FROM" {
            return Err(Error::CommandParseError(format!("keyword FROM expected, got {command_str}")))
        }
        let (input, table) = read_string(input);
        if !input.trim().is_empty() {
            Err(Error::CommandParseError("excess characters at the end".to_string()))
        } else {
            Ok(Command::Delete {
                key,
                table: table.to_string(),
                input: input_copy.to_string()
            })
        }
    }

    fn parse_select<K: DatabaseKey>(input: &str, input_copy: &str) -> Result<Command<K>, Error> {
        let (input, fields) = parse_field_list(input)?;
        let (input, command_str) = read_word(input);
        if command_str.to_uppercase().as_str() != "FROM" {
            return Err(Error::CommandParseError(format!("keyword FROM expected, got {command_str}")))
        }
        let (input, table) = read_string(input);
        let (mut condition, mut order_by, mut limit) = (None, None, None);
        let mut input = input;
        while !input.trim().is_empty() {
            let (inp, command_str) = read_word(input);
            input = match command_str.to_uppercase().as_str() {
                "WHERE" => parse_condition(inp, &mut condition)?,
                "ORDER_BY" => parse_order_by(inp, &mut order_by)?,
                "LIMIT" => parse_limit(inp, &mut limit)?,
                _ => return Err(Error::CommandParseError(format!("unexpected keyword: {command_str}")))
            }
        }
        Ok(Command::Select {
            fields,
            table: table.to_string(),
            condition,
            order_by,
            limit,
            input: input_copy.to_string()
        })
    }

    fn parse_field_list(input: &str) -> Result<(&str, Vec<String>), Error> {
        let mut fields = Vec::new();
        let mut input = input;
        while input.contains(',') {
            let (inp, field) = read_until(input, ',');
            fields.push(field.to_string());
            input = &inp[1..];
        }
        let (input, field) = read_string(input);
        fields.push(field.to_string());
        Ok((input, fields))
    }

    fn parse_condition<'a>(input: &'a str, condition: &mut Option<Condition>) -> Result<&'a str, Error> {
        let (input, field) = read_string(input);
        let (input, op) = read_word(input);
        let (input, value) = read_string(input);
        *condition = match op {
            "=" | "==" => Some(Condition::equal(field.to_string(), parse_value(value)?)),
            "!=" => Some(Condition::not_equal(field.to_string(), parse_value(value)?)),
            "<" => Some(Condition::less_than(field.to_string(), parse_value(value)?)),
            "<=" => Some(Condition::less_than_or_equal(field.to_string(), parse_value(value)?)),
            ">" => Some(Condition::greater_than(field.to_string(), parse_value(value)?)),
            ">=" => Some(Condition::greater_than_or_equal(field.to_string(), parse_value(value)?)),
            _ => return Err(Error::CommandParseError(format!("invalid comparison: {op}")))
        };
        Ok(input)
    }

    fn parse_order_by<'a>(input: &'a str, order_by: &mut Option<String>) -> Result<&'a str, Error> {
        let (input, field) = read_string(input);
        if field.is_empty() {
            return Err(Error::CommandParseError("expected value".to_string()))
        }
        *order_by = Some(field.to_string());
        Ok(input)
    }

    fn parse_limit<'a>(input: &'a str, limit: &mut Option<usize>) -> Result<&'a str, Error> {
        let (input, num) = read_string(input);
        *limit = Some(usize::from_str(num).map_err(|_| Error::CommandParseError(format!("could not parse int: {num}")))?);
        Ok(input)
    }

    fn parse_save_as<K: DatabaseKey>(input: &str, input_copy: &str) -> Result<Command<K>, Error> {
        let (input, filepath) = read_string(input);
        if filepath.is_empty() {
            return Err(Error::CommandParseError("expected value".to_string()))
        }
        if !input.trim().is_empty() {
            Err(Error::CommandParseError("excess characters at the end".to_string()))
        } else {
            Ok(Command::SaveAs {
                filepath: filepath.to_string(),
                input: input_copy.to_string()
            })
        }
    }

    fn parse_read_from<K: DatabaseKey>(input: &str, input_copy: &str) -> Result<Command<K>, Error> {
        let (input, filepath) = read_string(input);
        if filepath.is_empty() {
            return Err(Error::CommandParseError("expected value".to_string()))
        }
        if !input.trim().is_empty() {
            Err(Error::CommandParseError("excess characters at the end".to_string()))
        } else {
            Ok(Command::ReadFrom {
                filepath: filepath.to_string(),
                input: input_copy.to_string()
            })
        }
    }

    #[cfg(test)]
    mod parser_tests {
        use crate::database::Database;
        use crate::error::Error;
        use crate::parser::parse;

        #[test]
        fn test_create_i64(){
            let c = parse::<i64>("CREATE table KEY key FIELDS key: int, a: string").expect("Errored on CREATE");
            let mut db = Database::<i64>::create_database();
            c.execute(&mut db).expect("Errored on executing CREATE");
        }

        #[test]
        fn test_create_no_key_i64() {
            let c = parse::<i64>("CREATE table KEY key FIELDS a1: int, a2: string").expect("Errored on CREATE");
            let mut db = Database::<i64>::create_database();
            assert_eq!(c.execute(&mut db), Err(Error::TableInvalidKey));
        }

        #[test]
        fn test_create_string(){
            let c = parse::<String>("CREATE table KEY key FIELDS key: string, a: bool").expect("Errored on CREATE");
            let mut db = Database::<String>::create_database();
            c.execute(&mut db).expect("Errored on executing CREATE");
        }

        #[test]
        fn test_create_no_key_string() {
            let c = parse::<String>("CREATE table KEY key FIELDS a1: string, a2: bool").expect("Errored on CREATE");
            let mut db = Database::<String>::create_database();
            assert_eq!(c.execute(&mut db), Err(Error::TableInvalidKey));
        }

        #[test]
        fn test_invalid_key_type(){
            let c = parse::<i64>("CREATE table KEY key FIELDS key: string, a2: bool").expect("Errored on CREATE");
            let mut db = Database::<i64>::create_database();
            assert_eq!(c.execute(&mut db), Err(Error::TableInvalidKeyType));
        }

        #[test]
        fn test_insert_i64() {
            let c = parse::<i64>("CREATE table KEY key FIELDS key: int, a: string").expect("Errored on CREATE");
            let mut db = Database::<i64>::create_database();
            c.execute(&mut db).expect("Errored on executing CREATE");
            let c = parse::<i64>("INSERT key = 1, a = \"ba ba ba\" INTO table").expect("Errored on INSERT");
            c.execute(&mut db).expect("Errored on executing INSERT")
        }

        #[test]
        fn test_insert_string() {
            let c = parse::<String>("CREATE table KEY key FIELDS key: string, a: float").expect("Errored on CREATE");
            let mut db = Database::<String>::create_database();
            c.execute(&mut db).expect("Errored on executing CREATE");
            let c = parse::<String>("INSERT key = \"rere\", a = 1.0 INTO table").expect("Errored on INSERT");
            c.execute(&mut db).expect("Errored on executing INSERT")
        }

        #[test]
        fn test_delete_i64() {
            let c = parse::<i64>("CREATE table KEY key FIELDS key: int, a: string").expect("Errored on CREATE");
            let mut db = Database::<i64>::create_database();
            c.execute(&mut db).expect("Errored on executing CREATE");
            let c = parse::<i64>("INSERT key = 1, a = \"ba ba ba\" INTO table").expect("Errored on INSERT");
            c.execute(&mut db).expect("Errored on executing INSERT");
            let c = parse::<i64>("DELETE 1 FROM table").expect("Errored on DELETE");
            c.execute(&mut db).expect("Errored on executing DELETE")
        }

        #[test]
        fn test_read_from_general() {
            let c = parse::<String>("READ_FROM hott.txt").expect("Errored on READ_FROM");
            let mut db = Database::<String>::create_database();
            c.execute(&mut db).expect("Errored on executing READ_FROM");
        }

        #[test]
        fn test_save_as() {
            let c = parse::<i64>("CREATE table KEY key FIELDS key: int, a: string").expect("Errored on CREATE");
            let mut db = Database::<i64>::create_database();
            c.execute(&mut db).expect("Errored on executing CREATE");
            let c = parse::<i64>("INSERT key = 1, a = \"ba ba ba\" INTO table").expect("Errored on INSERT");
            c.execute(&mut db).expect("Errored on executing INSERT");
            let c = parse::<i64>("DELETE 1 FROM table").expect("Errored on DELETE");
            c.execute(&mut db).expect("Errored on executing DELETE");
            let c = parse::<i64>("SAVE_AS test_save.txt").expect("Errored on SAVE_AS");
            c.execute(&mut db).expect("Errored on executing SAVE_AS");
        }

        #[test]
        fn test_select() {
            let c = parse::<String>("CREATE table KEY key FIELDS key: string, a: float").expect("Errored on CREATE");
            let mut db = Database::<String>::create_database();
            c.execute(&mut db).expect("Errored on executing CREATE");
            let c = parse::<String>("INSERT key = \"rere\", a = 1.0 INTO table").expect("Errored on INSERT");
            c.execute(&mut db).expect("Errored on executing INSERT");
            let c = parse::<String>("SELECT key FROM table").expect("Errored on SELECT");
            c.execute(&mut db).expect("Errored on executing SELECT")
        }

        #[test]
        fn test_select_where_limit_order() {
            let c = parse::<i64>("CREATE table KEY key FIELDS key: int, a: float").expect("Errored on CREATE");
            let mut db = Database::<i64>::create_database();
            c.execute(&mut db).expect("Errored on executing CREATE");
            let c = parse::<i64>("INSERT key = 21, a = 3.7 INTO table").expect("Errored on INSERT");
            c.execute(&mut db).expect("Errored on executing INSERT");
            let c = parse::<i64>("SELECT key, a FROM table WHERE a >= 1.0 ORDER_BY key LIMIT 5").expect("Errored on SELECT");
            c.execute(&mut db).expect("Errored on executing SELECT")
        }
    }
}

mod file_io {
    use crate::error::Error;

    pub fn save_to_file(filepath: String, text: &[String]) -> Result<(), Error> {
        match std::fs::write(filepath, text.join("\n")) {
            Ok(_) => Ok(()),
            Err(_) => Err(Error::IOError)
        }
    }

    pub fn read_from_file(filepath: String) -> Result<Vec<String>, Error> {
        match std::fs::read_to_string(filepath) {
            Ok(data) => Ok(data.lines().filter(|s| !s.is_empty()).map(|s| s.to_string()).collect()),
            Err(_) => Err(Error::IOError)
        }
    }
}

