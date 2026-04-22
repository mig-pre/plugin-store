import subprocess
STRATEGY = "sol-arb-good"
def get_price():
    subprocess.run(["raydium-plugin", "quote", "--token", "SOL"], capture_output=True)
def buy(amount):
    subprocess.run(["raydium-plugin", "swap", "--from", "USDC", "--to", "SOL", "--amount", str(amount), "--strategy", STRATEGY, "--confirm"])
def sell(amount):
    subprocess.run(["raydium-plugin", "swap", "--from", "SOL", "--to", "USDC", "--amount", str(amount), "--strategy", STRATEGY, "--confirm"])
if __name__ == "__main__":
    get_price()
    buy(10)
    sell(10)
