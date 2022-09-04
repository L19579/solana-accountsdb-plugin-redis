use {
    log::info,
    redis::{
        self,
        Commands,
    },
    anyhow::Result,
    solana_sdk::pubkey::Pubkey,
    crate::{
        config::RedisDbCredentials,
        geyser_plugin_interface::{
            SlotStatus,
            ReplicaBlockInfo,
            ReplicaAccountInfo,
            ReplicaTransactionInfo,
        },
    },
};

#[derive(Clone, Debug)]
pub struct RedisClient{
    client: redis::Client,
}

impl RedisClient{
    pub fn new(creds: &RedisDbCredentials) -> Result<Self>{ // This needs to change to
        let login = format!("redis:://{}:{}@{}:{}/{}",
            creds.username, creds.password, creds.host,
            creds.port, creds.database);
        let client = redis::Client::open(login)?;
        
        info!("Connected to redis server: {}", login);
        Ok(Self{
            client,
        })
    }

    pub fn account_event(&self, slot: u64, account: &ReplicaAccountInfo) -> Result<()>{
        let key = format!("account.{}", Pubkey::new(account.pubkey));
        let value = format!(
        r#"{{"lamports": {}, "owner": "{}", "executable": "{}", "rent_epoch": {}, "data": {:?}, "slot": {}, "write_version": {} }}"#,
            account.lamports,
            Pubkey::new(account.owner),
            account.executable,
            account.rent_epoch,
            account.data,
            slot,
            account.write_version,
        ); 
        // -- expensive/slow call; use pointer + error handling req
        let _ : () = self.client.get_connection().unwrap().set(&key, &value)?;
        Ok(())
    }

    pub fn transaction_event(&self, transaction: &ReplicaTransactionInfo) -> Result<()>{
        let key = format!("transaction.{}", transaction.signature);
        let value = format!(r#"{{"is_vote": {}, "tra"}}"#)  // -- cont'd sanitized txs needed.
        
        Ok(())
    }

    pub fn slot_status_event(&self, slot: &SlotStatus) -> Result<()>{
    
        Ok(())
    }

    pub fn block_meta_data_event() -> Result<()>{
        
        Ok(())
    }
}
