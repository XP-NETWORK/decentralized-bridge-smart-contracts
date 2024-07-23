import ICRC7 "mo:icrc7-mo";
import Text "mo:base/Text";

module{
  public let defaultConfig = func(caller: Principal, name: ?Text, symbol: ?Text) : ICRC7.InitArgs{
      ?{
        symbol = symbol;
        name = name;
        description = ?"A Collection of Nebulas Captured by NASA";
        logo = ?"https://www.nasa.gov/wp-content/themes/nasa/assets/images/nasa-logo.svg";
        supply_cap = null;
        allow_transfers = null;
        max_query_batch_size = ?100;
        max_update_batch_size = ?100;
        default_take_value = ?1000;
        max_take_value = ?10000;
        max_memo_size = ?512;
        permitted_drift = null;
        tx_window = null;
        burn_account = null; //burned nfts are deleted
        deployer = caller;
        owner = caller;
        supported_standards = null;
      };
  };
};