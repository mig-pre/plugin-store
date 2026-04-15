package main

import (
	"encoding/json"
	"flag"
	"fmt"
	"net/http"
	"os"
	"strconv"
	"strings"
	"time"
)

var version = "0.1.0"

type OkxResponse struct {
	Code string      `json:"code"`
	Data []OkxTicker `json:"data"`
}

type OkxTicker struct {
	Last      string `json:"last"`
	Open24h   string `json:"open24h"`
	High24h   string `json:"high24h"`
	Low24h    string `json:"low24h"`
	VolCcy24h string `json:"volCcy24h"`
}

type PriceOutput struct {
	OK           bool   `json:"ok"`
	Token        string `json:"token"`
	PriceUSD     string `json:"price_usd"`
	Open24h      string `json:"open_24h"`
	High24h      string `json:"high_24h"`
	Low24h       string `json:"low_24h"`
	Volume24h    string `json:"volume_24h"`
	Change24hPct string `json:"change_24h_pct"`
	Source       string `json:"source"`
}

func getPrice(token string) error {
	symbol := strings.ToUpper(token)
	url := fmt.Sprintf("https://www.okx.com/api/v5/market/ticker?instId=%s-USDT", symbol)

	client := &http.Client{Timeout: 10 * time.Second}
	resp, err := client.Get(url)
	if err != nil {
		return fmt.Errorf("failed to call OKX API: %w", err)
	}
	defer resp.Body.Close()

	var okxResp OkxResponse
	if err := json.NewDecoder(resp.Body).Decode(&okxResp); err != nil {
		return fmt.Errorf("failed to parse response: %w", err)
	}

	if okxResp.Code != "0" || len(okxResp.Data) == 0 {
		return fmt.Errorf("OKX API error: code=%s (token %s-USDT may not exist)", okxResp.Code, symbol)
	}

	ticker := okxResp.Data[0]

	// Calculate 24h change percentage
	changePct := "N/A"
	if last, err1 := strconv.ParseFloat(ticker.Last, 64); err1 == nil {
		if open, err2 := strconv.ParseFloat(ticker.Open24h, 64); err2 == nil && open > 0 {
			changePct = fmt.Sprintf("%.2f", (last-open)/open*100)
		}
	}

	output := PriceOutput{
		OK:           true,
		Token:        symbol,
		PriceUSD:     ticker.Last,
		Open24h:      ticker.Open24h,
		High24h:      ticker.High24h,
		Low24h:       ticker.Low24h,
		Volume24h:    ticker.VolCcy24h,
		Change24hPct: changePct,
		Source:       "okx-public-api",
	}

	data, err := json.MarshalIndent(output, "", "  ")
	if err != nil {
		return fmt.Errorf("failed to marshal output: %w", err)
	}

	fmt.Println(string(data))
	return nil
}

func main() {
	if len(os.Args) > 1 && os.Args[1] == "--version" {
		fmt.Printf("go-price-demo %s\n", version)
		os.Exit(0)
	}

	if len(os.Args) < 2 || os.Args[1] != "get-price" {
		fmt.Fprintf(os.Stderr, "Usage: go-price-demo get-price --token <SYMBOL>\n")
		os.Exit(1)
	}

	cmd := flag.NewFlagSet("get-price", flag.ExitOnError)
	token := cmd.String("token", "ETH", "Token symbol (e.g. ETH, BTC, SOL)")
	cmd.Parse(os.Args[2:])

	if err := getPrice(*token); err != nil {
		errOut, _ := json.MarshalIndent(map[string]interface{}{
			"ok":    false,
			"error": err.Error(),
		}, "", "  ")
		fmt.Println(string(errOut))
		os.Exit(1)
	}
}
