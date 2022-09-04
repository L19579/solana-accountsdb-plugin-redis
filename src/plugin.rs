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

#[derive(Debug, Clone)]
pub struct GeyserRedisPlugin{
    config: Option<GeyserConfig>,
    redis_client: Option<RedisClient>,
    target_accounts: Vec<[u8; 32]>,
    account_notifications: bool,
    transaction_notifications: bool,
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
        self.config = Some(config);
        self.redis_client = Some(match RedisClient::new(&config.redis_db_credentials){
            Ok(r_c) => r_c,
            Err(e) => {
               return  Err(GeyserPluginError::ConfigFileReadError{
                    msg: String::from(format!("{}", e)), // < -- drop format
                });
            }
        });
        self.account_notifications = config.account_data_notifications_enabled;
        self.transaction_notifications = config.transaction_data_notifications_enabled;

        // faster to compare bytes
        config.accounts.iter().for_each(|account|{
            let mut acc_bytes = [0u8; 32];
            acc_bytes.copy_from_slice(&bs58::decode(account)
                                      .into_vec()
                                      .unwrap()[0..32]);
            self.target_accounts.push(acc_bytes);
        });

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
        let account = match account{
            ReplicaAccountInfoVersions::V0_0_1(a) => a, 
        };

        self.target_accounts.iter().for_each(|target_account|{
            if target_account == account.pubkey {
                self.redis_client.unwrap().account_event(&account); // -- err handling req
            } 
        });
        Ok(()) 
    }
    
    fn update_slot_status(
        &mut self,
        slot: u64,
        parent: Option<u64>,
        status: SlotStatus
    ) -> GeyserResult<()> {
        // --
        Ok(())
    }

    fn notify_end_of_startup(&mut self) -> GeyserResult<()> {
        info!("End of Geyser Plugin Startup");
        Ok(())
    }

    fn notify_transaction(
        &mut self,
        transaction: ReplicaTransactionInfoVersions<'_>,
        slot: u64
    ) -> GeyserResult<()> {
        //--
        Ok(())
    }

    fn notify_block_metadata(
        &mut self,
        blockinfo: ReplicaBlockInfoVersions<'_>
    ) -> GeyserResult<()> {
        // -- 
        Ok(())
    }
    fn account_data_notifications_enabled(&self) -> bool {
        self.account_notifications
    }
    fn transaction_notifications_enabled(&self) -> bool {
        self.transaction_notifications
    }
    
}
