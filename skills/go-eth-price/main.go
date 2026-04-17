package main

import (
	"encoding/json"
	"fmt"
	"net/http"
	"os"
	"strconv"
	"time"
)

type OkxResponse struct {
	Data []struct {
		Last    string `json:"last"`
		Open24h string `json:"open24h"`
		Vol24h  string `json:"vol24h"`
	} `json:"data"`
}

func main() {
	resp, err := http.Get("https://www.okx.com/api/v5/market/ticker?instId=ETH-USDT")
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error: %v\n", err)
		os.Exit(1)
	}
	defer resp.Body.Close()

	var okx OkxResponse
	if err := json.NewDecoder(resp.Body).Decode(&okx); err != nil {
		fmt.Fprintf(os.Stderr, "Error: %v\n", err)
		os.Exit(1)
	}

	if len(okx.Data) == 0 {
		fmt.Fprintln(os.Stderr, "Error: No data")
		os.Exit(1)
	}

	t := okx.Data[0]
	price, _ := strconv.ParseFloat(t.Last, 64)
	open, _ := strconv.ParseFloat(t.Open24h, 64)
	change := ((price - open) / open) * 100

	out := map[string]interface{}{
		"token":      "ETH",
		"price_usd":  fmt.Sprintf("%.2f", price),
		"change_24h": fmt.Sprintf("%+.2f%%", change),
		"volume_24h": t.Vol24h,
		"timestamp":  time.Now().Unix(),
	}

	enc := json.NewEncoder(os.Stdout)
	enc.SetIndent("", "  ")
	enc.Encode(out)
}
