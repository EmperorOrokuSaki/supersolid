// This is a generated Motoko binding.
// Please use `import service "ic:canister_id"` instead to call canisters on the IC if possible.

module {
  public type HttpOutcallError = {
    #IcError : { code : RejectionCode; message : Text };
    #InvalidHttpJsonRpcResponse : {
      status : Nat16;
      body : Text;
      parsingError : ?Text;
    };
  };
  public type JsonRpcError = { code : Int64; message : Text };
  public type ProviderError = {
    #TooFewCycles : { expected : Nat; received : Nat };
    #MissingRequiredProvider;
    #ProviderNotFound;
    #NoPermission;
  };
  public type RejectionCode = {
    #NoError;
    #CanisterError;
    #SysTransient;
    #DestinationInvalid;
    #Unknown;
    #SysFatal;
    #CanisterReject;
  };
  public type Result = { #Ok : {}; #Err : RouterError };
  public type RouterError = { #Rpc : RpcError; #Locked; #Unknown : Text };
  public type RpcError = {
    #JsonRpcError : JsonRpcError;
    #ProviderError : ProviderError;
    #ValidationError : ValidationError;
    #HttpOutcallError : HttpOutcallError;
  };
  public type ValidationError = {
    #CredentialPathNotAllowed;
    #HostNotAllowed : Text;
    #CredentialHeaderNotAllowed;
    #UrlParseError : Text;
    #Custom : Text;
    #InvalidHex : Text;
  };
  public type Self = actor {
    balance : shared query Nat64 -> async Text;
    get_chain_ledger : shared query Nat64 -> async [(Text, [(?Text, Text)])];
    get_user_balance : shared query (Nat64, ?Text, Text) -> async Text;
    public_key : shared query () -> async Text;
    send_request : shared (Nat64, Text, Text, Nat) -> async Result;
    start : shared (Principal, [(Text, Nat64)]) -> async ();
  }
}