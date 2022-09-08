use {
    //anyhow::GeyserResult,
    bs58,
    log::info,
    crate::{
        geyser_plugin_interface::{
            Result as GeyserResult, 
            GeyserPluginError,
            SlotStatus,
            GeyserPlugin,
            ReplicaAccountInfo,
            ReplicaAccountInfoVersions,
            ReplicaTransactionInfo,
            ReplicaTransactionInfoVersions,
            ReplicaBlockInfo,
            ReplicaBlockInfoVersions,
        },
        client::RedisClient,
        config::{
            self,
            GeyserConfig, 
        },
    },
};

// as_refs are annoying, review need for Options
#[derive(Debug, Clone)]
pub struct GeyserRedisPlugin{
    pub config: Option<GeyserConfig>,
    pub redis_client: Option<RedisClient>,
    pub target_accounts: Vec<[u8; 32]>,
}

impl GeyserPlugin for GeyserRedisPlugin{

    fn name (&self) -> &'static str {
        "GeyserRedisPlugin"
    }

    fn on_load(&mut self, _config_file: &str) -> GeyserResult<()> {
        let config = match GeyserConfig::load(_config_file){
            Ok(c) => c,
            Err(e) => {
                return Err(GeyserPluginError::ConfigFileReadError{
                    msg: String::from("Error opening, or reading config file"),
                });
            }
        };
        self.redis_client = Some(match RedisClient::new(&config.redis_db_credentials){
            Ok(r_c) => r_c,
            Err(e) => {
               return  Err(GeyserPluginError::ConfigFileReadError{
                    msg: String::from(format!("{}", e)), // < -- drop format
                });
            }
        });

        // faster to compare bytes
        match config.accounts.as_ref() {
            Some(accounts) => {
                accounts.iter().for_each(|account|{
                    let mut acc_bytes = [0u8; 32];
                    acc_bytes.copy_from_slice(&bs58::decode(account)
                                              .into_vec()
                                              .unwrap()[0..32]);
                    self.target_accounts.push(acc_bytes);
                });
            },
            None => ()
        }

        self.config = Some(config);
        Ok(())
    }
    
    fn on_unload(&mut self) {
        // -- close redis server here
    }

    fn update_account(
        &mut self,
        account: ReplicaAccountInfoVersions<'_>,
        slot: u64,
        is_startup: bool
    ) -> GeyserResult<()> {
        if is_startup{
            return Ok(());
        }

        let account = match account{
            ReplicaAccountInfoVersions::V0_0_1(a) => a, 
            //V0_0_2 not supported on 1.10.x
            _ => {
                return Ok(());
            }
        };

        self.target_accounts.iter().for_each(|target_account|{
            if target_account == account.pubkey {
                self.redis_client.as_mut().unwrap() 
                    .account_event(slot, &account); // -- err handling req
            } 
        });
        Ok(()) 
    }
    
    fn notify_transaction(
        &mut self,
        transaction: ReplicaTransactionInfoVersions<'_>,
        slot: u64
    ) -> GeyserResult<()> {
        let config = self.config.as_ref().unwrap();
        let transaction = match transaction {
            ReplicaTransactionInfoVersions::V0_0_1(tx) => {
                if config.ignore_vote_transactions
                    .unwrap_or(true) && tx.is_vote{
                        return Ok(());
                    }
                tx
            },
            _ => {
                return Ok(());
            }
        };
        self.redis_client.as_ref().unwrap()
            .transaction_event(slot, &transaction); // -- err handling req
        Ok(())
    }
    
    fn update_slot_status(
        &mut self,
        slot: u64,
        parent: Option<u64>,
        status: SlotStatus
    ) -> GeyserResult<()> {
        self.redis_client.as_mut().unwrap()
            .slot_status_event(slot, parent, status); // -- err handling req
        Ok(())
    }

    fn notify_block_metadata(
        &mut self,
        blockinfo: ReplicaBlockInfoVersions<'_>
    ) -> GeyserResult<()> {
        let blockinfo = match blockinfo {
            ReplicaBlockInfoVersions::V0_0_1(b) => b, 
        };

        self.redis_client.as_mut().unwrap()
            .block_metadata_event(blockinfo); // --
        Ok(())
    }

    fn notify_end_of_startup(&mut self) -> GeyserResult<()> {
        info!("Geyser Plugin startup loaded");
        Ok(())
    }

    fn account_data_notifications_enabled(&self) -> bool {
        self.config.as_ref().unwrap()
            .account_data_notifications_enabled
            .unwrap_or(true)
    }

    fn transaction_notifications_enabled(&self) -> bool {
        self.config.as_ref().unwrap()
            .transaction_data_notifications_enabled
            .unwrap_or(false)
    }
    
}
