package main

import (
	"bytes"
	"encoding/binary"
	"encoding/xml"
	"errors"
	"fmt"
	"io"
	"io/ioutil"
	"net"
	"os"
	"sync"
	"time"
)

//MsgHead defines a
//generic head structure
type MsgHead struct {
	_msgType    uint32
	_bodyLength uint32
}

//Stock defines a struct
type Stock struct {
	sync.RWMutex
	code         string
	marginStatus uint32
}

func (p *Stock) setMargin(v uint32) {
	p.Lock()
	p.marginStatus = v
	p.Unlock()
}

//Stocks defines a thread-safe stock list
type Stocks struct {
	sync.RWMutex
	stocks map[string]*Stock
}

func newStocks() *Stocks {
	stocks := &Stocks{
		stocks: make(map[string]*Stock),
	}

	return stocks
}

func (p *Stocks) getOrDefaultStock(code string) *Stock {
	p.RLock()
	stock, ok := p.stocks[code]
	p.RUnlock()

	if !ok {
		stock = &Stock{
			code: code,
		}
		p.Lock()
		p.stocks[code] = stock
		p.Unlock()
	}

	return stock
}

var stocks = newStocks()

func struct2Bytes(msg interface{}) ([]byte, error) {
	buf := bytes.NewBuffer(nil)

	err := binary.Write(buf, binary.BigEndian, msg)

	if err != nil {
		return nil, err
	}

	return buf.Bytes(), nil
}

func generateCheckSum(buf []byte) uint32 {
	var sum uint64
	for _, b := range buf {
		sum += uint64(b)
	}

	return uint32(sum % 256)
}

//LoginMsgID ..
var LoginMsgID uint32 = 1

//SendCompID ..
var SendCompID = "F000648Q0011"

//TargetCompID ..
var TargetCompID = "VDE"

//HeartBeatInt ..
var HeartBeatInt uint32 = 20

//Password ..
var Password = "F000648Q0011"

//DefaultAppVerID ..
var DefaultAppVerID = "1.00"

func logon(conn net.Conn) error {
	type LogonMsg struct {
		header       MsgHead
		sendCompID   [20]byte
		targetCompID [20]byte
		heartBeat    uint32
		password     [16]byte
		version      [32]byte
		checksum     uint32
	}

	msg := LogonMsg{}
	msg.header._msgType = 1
	msg.header._bodyLength = 92

	copy(msg.sendCompID[:], []byte(SendCompID))
	copy(msg.targetCompID[:], []byte(TargetCompID))

	msg.heartBeat = HeartBeatInt
	copy(msg.password[:], []byte(Password))
	copy(msg.version[:], []byte(DefaultAppVerID))

	buf, err := struct2Bytes(&msg)
	if err != nil {
		return err
	}

	checksum := generateCheckSum(buf[:len(buf)-4])

	binary.BigEndian.PutUint32(buf[len(buf)-4:], checksum)

	n, err := conn.Write(buf)
	if err != nil {
		fmt.Println("logon: ", err)
		return err
	}

	if n != 104 {
		return errors.New("write failed")
	}

	return nil
}

func getMessage(conn net.Conn) (uint32, uint32, []byte, error) {
	var msgType uint32
	var bodyLen uint32
	err := binary.Read(conn, binary.BigEndian, &msgType)
	if err != nil {
		fmt.Println(err)
		return 0, 0, nil, err
	}

	err = binary.Read(conn, binary.BigEndian, &bodyLen)
	if err != nil {
		return 0, 0, nil, err
	}

	var buf []byte
	if bodyLen > 0 {
		buf = make([]byte, bodyLen)

		got := 0
		for {
			n, err2 := conn.Read(buf[got:])
			if err2 != nil || n < 0 {
				return 0, 0, nil, err2
			}

			got += n

			if got < int(bodyLen) {
				continue
			}

			break
		}

	}

	var checksum uint32
	err2 := binary.Read(conn, binary.BigEndian, &checksum)
	if err2 != nil {
		return 0, 0, nil, err2
	}

	//fmt.Println(msgType, bodyLen, buf)
	return msgType, checksum, buf, nil
}

func handleChannelHeartBeat(conn net.Conn, messageBody []byte) {

}

func handleResentMsg(conn net.Conn, messageBody []byte) {

}

func handleUserReport(conn net.Conn, messageBody []byte) {
	type reportMsg struct {
		origTime    int64
		versionCode [16]byte
		userNum     uint16
	}

	if len(messageBody) >= 26 {
		msg := &reportMsg{}
		msg.origTime = int64(binary.BigEndian.Uint64(messageBody))
		copy(msg.versionCode[:], messageBody[8:])
		msg.userNum = binary.BigEndian.Uint16(messageBody[24:])
		fmt.Println("ReportMessage: ", msg)
	} else {
		fmt.Println("Wrong message", len(messageBody))
	}
}

func handleChannelStatistic(conn net.Conn, messageBody []byte) {
	//do nothing about snapshot statistic message
}

func handleRealtimeStatus(conn net.Conn, messageBody []byte) {
	//uint32
	type Switch struct {
		switchType   uint16
		switchStatus uint16
	}

	type realState struct {
		origTime         int64
		channelNo        uint16
		securityID       [8]byte
		securityIDSource [4]byte //102 shenzhen, 103 hongkong
		financialStatus  [8]byte
		switchers        []Switch
	}

	msg := &realState{}

	msg.origTime = int64(binary.BigEndian.Uint64(messageBody))
	msg.channelNo = binary.BigEndian.Uint16(messageBody[8:])

	copy(msg.securityID[:], messageBody[10:18])
	copy(msg.securityIDSource[:], messageBody[18:22])
	copy(msg.financialStatus[:], messageBody[22:30])

	num := binary.BigEndian.Uint32(messageBody[30:])

	if num > 0 {
		msg.switchers = make([]Switch, num)
		for i := 0; i < int(num); i++ {
			msg.switchers[i].switchType = binary.BigEndian.Uint16(messageBody[30+i*4:])
			msg.switchers[i].switchStatus = binary.BigEndian.Uint16(messageBody[30+i*4+2:])
		}
	}

	fmt.Println("Realtime status: ", msg)
}

func handleStockReport(conn net.Conn, messageBody []byte) {
	type stockReport struct {
		origTime      int64
		channelNO     uint16
		newsID        [8]byte
		headLine      [128]byte
		rawDataFormat [8]byte //TXT, PDF, DOC
		rawDataLength uint32
		rawData       []byte
	}

	msg := &stockReport{}

	msg.origTime = int64(binary.BigEndian.Uint64(messageBody))
	msg.channelNO = binary.BigEndian.Uint16(messageBody[8:])

	copy(msg.newsID[:], messageBody[10:18])
	copy(msg.headLine[:], messageBody[18:146])
	copy(msg.rawDataFormat[:], messageBody[146:154])
	msg.rawDataLength = binary.BigEndian.Uint32(messageBody[154:])
	msg.rawData = messageBody[158 : 158+msg.rawDataLength]

	fmt.Println("Stock report: ", msg)
}

func handleMarketStatus(conn net.Conn, messageBody []byte) {
	//hongkong market status update?
}

//SnapShot defines the common fields
type SnapShot struct {
	origTime         int64
	channelNo        uint16
	mdStreamID       [3]byte
	securityID       [8]byte
	securityIDSource [4]byte //102 shenzhen, 103 hongkong
	tradingPhaseCode [8]byte
	prevClosePx      int64
	numTrades        int64
	totalVolumeTrade int64
	totalValueTrade  int64
}

func handleStockSnapshot(conn net.Conn, messageBody []byte) {

	type mdEntry struct {
		mdEntryType  [2]byte
		mdEntryPx    int64
		mdEntrySize  int64
		mdPriceLevel uint16
		numOfOrders  int64
		noOrders     uint32
		orderQtys    []int64
	}

	type snapShot300111 struct {
		snapshot SnapShot
		//30011
		noMDEntries uint32
		entries     []mdEntry
	}
	msg := &snapShot300111{}
	msg.snapshot.origTime = int64(binary.BigEndian.Uint64(messageBody))
	msg.snapshot.channelNo = binary.BigEndian.Uint16(messageBody[8:])

	copy(msg.snapshot.mdStreamID[:], messageBody[10:13])
	copy(msg.snapshot.securityID[:], messageBody[13:21])
	copy(msg.snapshot.securityIDSource[:], messageBody[21:25])
	copy(msg.snapshot.tradingPhaseCode[:], messageBody[25:33])

	msg.snapshot.prevClosePx = int64(binary.BigEndian.Uint64(messageBody[33:41]))
	msg.snapshot.numTrades = int64(binary.BigEndian.Uint64(messageBody[41:49]))
	msg.snapshot.totalVolumeTrade = int64(binary.BigEndian.Uint64(messageBody[49:57]))
	msg.snapshot.totalValueTrade = int64(binary.BigEndian.Uint64(messageBody[57:65]))

	msg.noMDEntries = binary.BigEndian.Uint32(messageBody[65:69])

	msg.entries = make([]mdEntry, msg.noMDEntries)

	start := 69
	for i := 0; i < int(msg.noMDEntries); i++ {
		entry := &msg.entries[i]

		copy(entry.mdEntryType[:], messageBody[start:start+2])
		entry.mdEntryPx = int64(binary.BigEndian.Uint64(messageBody[start+2 : start+10]))
		entry.mdEntrySize = int64(binary.BigEndian.Uint64(messageBody[start+10 : start+18]))
		entry.mdPriceLevel = binary.BigEndian.Uint16(messageBody[start+18 : start+20])
		entry.numOfOrders = int64(binary.BigEndian.Uint64(messageBody[start+20 : start+28]))
		entry.noOrders = binary.BigEndian.Uint32(messageBody[start+28 : start+32])

		entry.orderQtys = make([]int64, entry.noOrders)
		pos := start + 32
		for j := 0; j < int(entry.noOrders); j++ {
			entry.orderQtys[j] = int64(binary.BigEndian.Uint64(messageBody[pos : pos+8]))
			pos += 8
		}
		start = pos
	}

	fmt.Println("Stocksnapshot: ", msg)
}

func handleIndexSnapshot(conn net.Conn, messageBody []byte) {
	type mdEntry struct {
		mdEntryType [2]byte
		mdEntryPx   int64
	}

	type snapShot309011 struct {
		snapshot SnapShot
		//309011
		noMDEntries uint32
		entries     []mdEntry
	}
	msg := &snapShot309011{}
	msg.snapshot.origTime = int64(binary.BigEndian.Uint64(messageBody))
	msg.snapshot.channelNo = binary.BigEndian.Uint16(messageBody[8:])

	copy(msg.snapshot.mdStreamID[:], messageBody[10:13])
	copy(msg.snapshot.securityID[:], messageBody[13:21])
	copy(msg.snapshot.securityIDSource[:], messageBody[21:25])
	copy(msg.snapshot.tradingPhaseCode[:], messageBody[25:33])

	msg.snapshot.prevClosePx = int64(binary.BigEndian.Uint64(messageBody[33:41]))
	msg.snapshot.numTrades = int64(binary.BigEndian.Uint64(messageBody[41:49]))
	msg.snapshot.totalVolumeTrade = int64(binary.BigEndian.Uint64(messageBody[49:57]))
	msg.snapshot.totalValueTrade = int64(binary.BigEndian.Uint64(messageBody[57:65]))

	msg.noMDEntries = binary.BigEndian.Uint32(messageBody[65:69])

	msg.entries = make([]mdEntry, msg.noMDEntries)

	start := 69
	for i := 0; i < int(msg.noMDEntries); i++ {
		entry := &msg.entries[i]

		copy(entry.mdEntryType[:], messageBody[start:start+2])
		entry.mdEntryPx = int64(binary.BigEndian.Uint64(messageBody[start+2 : start+10]))
		start += 10
	}

	fmt.Println("IndexSnapshot: ", msg)
}

func handleIndexVolumeStatistic(conn net.Conn, messageBody []byte) {

}

func handleMessage(conn net.Conn, msgType uint32, messageBody []byte) {

	switch msgType {
	case 3:
		if heartbeat(conn) != nil {
			fmt.Println("heartbeat failed")
		}
	case 390095:
		handleChannelHeartBeat(conn, messageBody)
	case 390094:
		handleResentMsg(conn, messageBody)
	case 390093:
		handleUserReport(conn, messageBody)
	case 390090:
		handleChannelStatistic(conn, messageBody)
	case 390019:
		handleMarketStatus(conn, messageBody)
	case 390013:
		handleRealtimeStatus(conn, messageBody)
	case 390012:
		handleStockReport(conn, messageBody)
	case 300111:
		handleStockSnapshot(conn, messageBody)
	case 300611:
		//060, 061, after trading snapshot
	case 306311:
		//hongkong stocks
	case 309011:
		handleIndexSnapshot(conn, messageBody)
	case 309111:
		handleIndexVolumeStatistic(conn, messageBody)
	default:
		fmt.Println(msgType, messageBody)
	}

}

func heartbeat(conn net.Conn) error {
	type heartbeatMsg struct {
		header   MsgHead
		checksum uint32
	}

	msg := heartbeatMsg{
		header: MsgHead{
			_msgType:    3,
			_bodyLength: 0,
		},
		checksum: 0,
	}

	bs, err := struct2Bytes(&msg)
	if err != nil {
		return err
	}
	checksum := generateCheckSum(bs[:len(bs)-4])

	binary.BigEndian.PutUint32(bs[len(bs)-4:], checksum)
	n, err2 := conn.Write(bs)
	if err2 != nil {
		return err2
	}

	if n != len(bs) {
		return errors.New("write failed")
	}
	return nil
}

type config struct {
	TargetAddr string `xml:"target_addr"`
}

func loadConfig() *config {
	content, err := ioutil.ReadFile("app.def")
	if err != nil {
		return nil
	}

	c := new(config)
	err = xml.Unmarshal(content, c)
	if err != nil {
		return nil
	}

	return c
}

func main() {
	config := loadConfig()
	if config == nil {
		fmt.Println("config error")
		return
	}

	conn, err := net.Dial("tcp", config.TargetAddr)
	//conn, err := net.Dial("tcp", "10.139.2.234:9016")
	if err != nil {
		fmt.Println("1:", err)
		return
	}
	defer conn.Close()

	err = logon(conn)
	if err != nil {
		fmt.Println(err)
		return
	}

	go func() {
		for {
			select {
			case <-time.After(time.Duration(5) * time.Second):
				if heartbeat(conn) != nil {
					fmt.Println("heartbeat failed")
					break
				}
			}
		}
	}()

	for {
		msgType, _, messageBody, err2 := getMessage(conn)
		if err2 != nil {
			fmt.Println(err2)
			break
		}
		fmt.Println(messageBody)
		handleMessage(conn, msgType, messageBody)
	}
}

func main2() {
	conn, err := net.Dial("tcp", "127.0.0.1:6666")

	if err != nil {
		fmt.Println("1:", err)
		return
	}
	defer conn.Close()

	file, err2 := os.OpenFile("binary", os.O_RDWR|os.O_CREATE|os.O_EXCL, 0600)
	if err2 != nil {
		fmt.Println("2:", err2)
		return
	}
	defer file.Close()

	var sum int64
	for {
		count, err3 := io.Copy(file, conn)
		if err3 != nil {
			fmt.Println("3:", err3)
			break
		}

		sum += count
		if sum > 10000 {
			break
		}
	}

}
