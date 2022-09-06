use {
    log::info,
    redis::{
        self,
        Commands,
    },
    anyhow::Result,
    solana_sdk::{
        pubkey::Pubkey,
        message::SanitizedMessage,
    },
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
    pub client: redis::Client,
}

impl RedisClient{
    pub fn new(creds: &RedisDbCredentials) -> Result<Self>{ 
        let login = format!("redis:://{}:{}@{}:{}/{}",
            creds.username, creds.password, creds.host,
            creds.port, creds.database);
        let client = redis::Client::open(login.clone())?;
        
        info!("Connected to redis server: {}", login);
        Ok(Self{
            client,
        })
    }

    pub fn account_event(&mut self, slot: u64, account: &ReplicaAccountInfo) -> Result<()>{
        let key = format!("account.{}", Pubkey::new(account.pubkey));
        redis::cmd("HSET").arg(&key)
            .arg("lamports").arg(account.lamports)
            .arg("owner").arg(Pubkey::new(account.owner).to_string())
            .arg("executable").arg(account.executable)
            .arg("rent_epoch").arg(account.rent_epoch)
            .arg("data").arg(account.data)
            .arg("write_version").arg(account.write_version)
            .arg("slot").arg(slot)
            .execute(&mut self.client.get_connection()?);
        Ok(())
    }

    pub fn transaction_event(&self, slot: u64, tx_event: &ReplicaTransactionInfo) -> Result<()>{
        let mut connection = self.client.get_connection()?;
        let key = format!("transaction.{}", tx_event.signature);
        let mut db_cmd = redis::Cmd::new();
        db_cmd.arg("HSET").arg(&key);
        db_cmd.arg("slot").arg(slot);
        db_cmd.arg("is_vote").arg(tx_event.is_vote);
       
        let tx = tx_event.transaction;
        db_cmd.arg("message_hash").arg(tx.message_hash().to_string());
        match tx.message(){
            SanitizedMessage::Legacy(message) => {

                // message header
                db_cmd.arg("message.header.num_required_signatures")
                    .arg(message.header.num_required_signatures)
                    .arg("message.header.num_readonly_signed_accounts")
                    .arg(message.header.num_readonly_signed_accounts)
                    .arg("message.header.num_readonly_unsigned_accounts")
                    .arg(message.header.num_readonly_unsigned_accounts);

                // message pubkeys
                let mut i_message_account = 0u8;
                message.account_keys.iter().for_each(|account_key|{
                    db_cmd.arg(&format!("message.account.index_{}", i_message_account)) 
                        .arg(account_key.to_string());
                    i_message_account += 1;
                });

                // recent blockhash used for message
                db_cmd.arg("message.recent_blockhash")
                    .arg(message.recent_blockhash.to_string());

                // message instructions and data
                let mut i_message_instruction = 0u8;
                message.instructions.iter().for_each(|instruction|{
                    let mut field_prefix = format!("message.instructions.index_{}", i_message_instruction);
                    db_cmd.arg(&format!("{}.program_id_index", field_prefix))
                        .arg(instruction.program_id_index)
                        .arg(&format!("{}.account_indices", field_prefix))
                        .arg(&instruction.accounts)
                        .arg(&format!("{}.data", field_prefix))
                        .arg(&instruction.data);
                    i_message_instruction += 1;
                });
            },
            SanitizedMessage::V0(message) => {
                // TODO: required for 1.11.x
            },
        };
        
        db_cmd.execute(&mut connection);
        Ok(())
    }

    pub fn slot_status_event(
        &self, slot: u64, 
        parent: Option<u64>, 
        status: SlotStatus) 
    -> Result<()>{
        let mut connection = self.client.get_connection()?;
        let key = format!("slot.{}", slot);
        let mut db_cmd = redis::Cmd::new();
        
        if parent.is_some(){ // -- clean up
            db_cmd.arg("parent")
                .arg(parent.unwrap());
        } 
        db_cmd.arg("status")
            .arg(status.as_str());

        db_cmd.execute(&mut connection);
        Ok(())
    }

    pub fn block_meta_data_event() -> Result<()>{
        
        Ok(())
    }
}
