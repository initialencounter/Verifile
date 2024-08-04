use redb::{Database, Error, ReadTransaction, TableDefinition, TransactionError, WriteTransaction};

const TABLE_PATH: TableDefinition<String, String> = TableDefinition::new("PATH_HASH");
const TABLE_HASH: TableDefinition<String, String> = TableDefinition::new("HASH_PATH");
const TABLE_DATE: TableDefinition<String, String> = TableDefinition::new("PATH_DATE");

pub(crate) struct NtfsDataBase {
    db: Database,
}

impl NtfsDataBase {
    pub fn new() -> Self {
        let db = Database::open("fileInfo.redb");
        match db { 
            Ok(db) => {
                NtfsDataBase { db }
            },
            Err(_err) => {
                let db = Database::create("fileInfo.redb").unwrap();
                let write_txn = db.begin_write().unwrap();
                write_txn.open_table(TABLE_PATH).unwrap();
                write_txn.open_table(TABLE_HASH).unwrap();
                write_txn.open_table(TABLE_DATE).unwrap();
                write_txn.commit().unwrap();
                println!("fileInfo.redb not found, created a new one");
                NtfsDataBase { db }
            }
        }
    }

    fn begin_read_txn(&self) -> Result<ReadTransaction, TransactionError> {
        self.db.begin_read()
    }

    fn begin_write_txn(&self) -> Result<WriteTransaction, TransactionError> {
        self.db.begin_write()
    }

    fn insert_value(
        &self,
        txn: &mut WriteTransaction,
        table_def: TableDefinition<String, String>,
        key: String,
        value: String,
    ) -> Result<(), Error> {
        let mut table = txn.open_table(table_def)?;
        table.insert(key, value)?;
        Ok(())
    }

    fn get_value(
        &self,
        txn: &ReadTransaction,
        table_def: TableDefinition<String, String>,
        key: String,
    ) -> Result<Option<String>, Error> {
        let table = txn.open_table(table_def)?;
        Ok(table.get(key)?.map(|v| v.value()))
    }

    pub fn insert_hash(&self, key: String, value: String) -> Result<(), Error> {
        let mut txn = self.begin_write_txn()?;
        self.insert_value(&mut txn, TABLE_PATH, key, value)?;
        txn.commit()?;
        Ok(())
    }

    pub fn get_hash(&self, key: String) -> Result<Option<String>, Error> {
        let txn = self.begin_read_txn()?;
        return  self.get_value(&txn, TABLE_PATH, key)
    }

    pub fn insert_path(&self, key: String, value: String) -> Result<(), Error> {
        let mut txn = self.begin_write_txn()?;
        let mut path = match self.get_path(key.clone()) { 
            Ok(Some(path)) => path,
            _ => String::from(""),
        };
        path = format!("{}\n{}", path, value);
        self.insert_value(&mut txn, TABLE_HASH, key, path)?;
        txn.commit()?;
        Ok(())
    }

    pub fn get_path(&self, key: String) -> Result<Option<String>, Error> {
        let txn = self.begin_read_txn()?;
        self.get_value(&txn, TABLE_HASH, key)
    }

    pub fn insert_date(&self, key: String, value: String) -> Result<(), Error> {
        let txn = self.begin_write_txn()?;
        txn.open_table(TABLE_DATE)?.insert(key, value.to_string())?;
        txn.commit()?;
        Ok(())
    }

    pub fn get_date(&self, key: String) -> Result<Option<String>, Error> {
        let txn = self.begin_read_txn()?;
        let table = txn.open_table(TABLE_DATE)?;
        Ok(table.get(key)?.map(|v| v.value()))
    }
}
