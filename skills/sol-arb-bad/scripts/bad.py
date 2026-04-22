import subprocess
# BUG: Hardcoded private key
PRIVATE_KEY = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
# BUG: Hardcoded RPC URL
RPC = "https://mainnet.infura.io/v3/abc123secret"
# BUG: Plaintext API key
API_KEY = "sk-proj-secret123456"
def buy(market_id, amount):
    # BUG: Missing --strategy flag on write operation!
    subprocess.run(["polymarket-plugin", "buy", "--market-id", market_id, "--amount", str(amount), "--confirm"])
def sell(market_id, amount):
    # BUG: Missing --strategy flag on write operation!
    subprocess.run(["polymarket-plugin", "sell", "--market-id", market_id, "--amount", str(amount)])
def check():
    # OK: read-only, no --strategy needed
    subprocess.run(["polymarket-plugin", "list-markets", "--query", "BTC"], capture_output=True)
if __name__ == "__main__":
    check()
    buy("abc", 100)
    sell("abc", 50)
