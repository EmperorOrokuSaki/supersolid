export interface Env {
	RPC_PROXY: DurableObjectNamespace;
  }
  
  type JsonRpcRequest = {
	jsonrpc: "2.0";
	method: string;
	params: any[];
	id: number;
  };
  
  const enum RpcProxyState {
	NotStarted,
	InProgress,
	Done,
  }
  
  export class RpcProxy {
	state: RpcProxyState = RpcProxyState.NotStarted;
	responseText?: string;
	responseStatus?: number;
	env: Env;
  
	constructor(state: DurableObjectState, env: Env) {
	  this.env = env;
	}
  
	async fetch(request: Request): Promise<Response> {
	  let json: JsonRpcRequest;
	  try {
		json = await request.json();
		if (json.jsonrpc !== "2.0" || !json.method || !json.params) {
		  throw new Error();
		}
	  } catch (e) {
		console.error(e);
		return new Response("Invalid JSON-RPC request", { status: 400 });
	  }
  
	  const alchemyApiKey = request.headers.get("Alchemy-Api-Key");
	  if (!alchemyApiKey) {
		return new Response("Alchemy API key is missing", { status: 400 });
	  }
  
	  const path = new URL(request.url).pathname;
	  const network = path.slice(1);
	  if (!network) {
		return new Response("Network not specified", { status: 404 });
	  }
  
	  return this.proxyRpcRequest(json, network, alchemyApiKey);
	}
  
	async proxyRpcRequest(
	  request: JsonRpcRequest,
	  chain: string,
	  alchemyApiKey: string,
	): Promise<Response> {
	  if (this.state === RpcProxyState.NotStarted) {
		this.state = RpcProxyState.InProgress;
		const url = `https://${chain}.g.alchemy.com/v2/${alchemyApiKey}`;
		try {
		  const response = await fetch(url, {
			method: "POST",
			headers: {
			  "Content-Type": "application/json",
			  "Accept": "application/json",
			},
			body: JSON.stringify(request),
		  });
		  this.responseText = await response.text();
		  this.responseStatus = response.status;
		} catch (e) {
		  console.error(e);
		  this.responseText = "Couldn't fetch data";
		  this.responseStatus = 500;
		}
		this.state = RpcProxyState.Done;
	  }
  
	  if (this.state === RpcProxyState.InProgress) {
		while (this.state === RpcProxyState.InProgress) {
		  await new Promise((resolve) => setTimeout(resolve, 100));
		}
	  }
  
	  return new Response(this.responseText, { status: this.responseStatus });
	}
  }
  
  export default {
	async fetch(
	  request: Request,
	  env: Env,
	  ctx: ExecutionContext,
	): Promise<Response> {
	  const path = new URL(request.url).pathname;
  
	  const id = env.RPC_PROXY.idFromName(path.slice(1) + request.headers.get("Alchemy-Api-Key"));
	  const stub = env.RPC_PROXY.get(id);
  
	  return stub.fetch(request);
	},
  };
  