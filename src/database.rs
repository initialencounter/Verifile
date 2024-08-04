use redb::{Database, Error, ReadTransaction, TableDefinition, TransactionError, WriteTransaction};

struct FileInfo {
    pub size: i64,
    pub last_accessed: i64,
}

const TABLE_PATH: TableDefinition<String, u32> = TableDefinition::new("PATH_CRC32");
const TABLE_CRC32: TableDefinition<u32, String> = TableDefinition::new("CRC32_PATH");
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
            Err(err) => {
                let db = Database::create("fileInfo.redb").unwrap();
                let write_txn = db.begin_write().unwrap();
                write_txn.open_table(TABLE_PATH).unwrap();
                write_txn.open_table(TABLE_CRC32).unwrap();
                write_txn.open_table(TABLE_DATE).unwrap();
                write_txn.commit().unwrap();
                panic!("Error: An unknown error occurred.{:?}", err)
            }
        }
    }

    fn begin_read_txn(&self) -> Result<ReadTransaction, TransactionError> {
        self.db.begin_read()
    }

    fn begin_write_txn(&self) -> Result<WriteTransaction, TransactionError> {
        self.db.begin_write()
    }

    fn insert_string_key_value(
        &self,
        txn: &mut WriteTransaction,
        table_def: TableDefinition<String, u32>,
        key: String,
        value: u32,
    ) -> Result<(), Error> {
        let mut table = txn.open_table(table_def)?;
        table.insert(key, value)?;
        Ok(())
    }

    fn insert_u32_key_value(
        &self,
        txn: &mut WriteTransaction,
        table_def: TableDefinition<u32, String>,
        key: u32,
        value: String,
    ) -> Result<(), Error> {
        let mut table = txn.open_table(table_def)?;
        table.insert(key, value)?;
        Ok(())
    }

    fn get_string_key_value(
        &self,
        txn: &ReadTransaction,
        table_def: TableDefinition<String, u32>,
        key: String,
    ) -> Result<Option<u32>, Error> {
        let table = txn.open_table(table_def)?;
        Ok(table.get(key)?.map(|v| v.value()))
    }

    fn get_u32_key_value(
        &self,
        txn: &ReadTransaction,
        table_def: TableDefinition<u32, String>,
        key: u32,
    ) -> Result<Option<String>, Error> {
        let table = txn.open_table(table_def)?;
        Ok(table.get(key)?.map(|v| v.value()))
    }

    pub fn insert_crc32(&self, key: String, value: u32) -> Result<(), Error> {
        let mut txn = self.begin_write_txn()?;
        self.insert_string_key_value(&mut txn, TABLE_PATH, key, value)?;
        txn.commit()?;
        Ok(())
    }

    pub fn get_crc32(&self, key: String) -> Result<Option<u32>, Error> {
        let txn = self.begin_read_txn()?;
        return  self.get_string_key_value(&txn, TABLE_PATH, key)
    }

    pub fn insert_path(&self, key: u32, value: String) -> Result<(), Error> {
        let mut txn = self.begin_write_txn()?;
        self.insert_u32_key_value(&mut txn, TABLE_CRC32, key, value)?;
        txn.commit()?;
        Ok(())
    }

    pub fn get_path(&self, key: u32) -> Result<Option<String>, Error> {
        let txn = self.begin_read_txn()?;
        self.get_u32_key_value(&txn, TABLE_CRC32, key)
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
