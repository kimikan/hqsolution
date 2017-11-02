package main

import (
	"bytes"
	"encoding/binary"
	"errors"
	"fmt"
	"io"
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
	for b := range buf {
		sum += uint64(int8(b))
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
		buf := make([]byte, bodyLen)

		n, err2 := conn.Read(buf)
		if err2 != nil {
			return 0, 0, nil, err2
		}

		if n != int(bodyLen) {
			fmt.Println(n, bodyLen)
			return msgType, 0, nil, errors.New("Wrong size expect")
		}
	}

	var checksum uint32
	err2 := binary.Read(conn, binary.BigEndian, &checksum)
	if err2 != nil {
		return 0, 0, nil, err2
	}

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
}

func handleMessage(conn net.Conn, msgType uint32, messageBody []byte) {

	switch msgType {
	case 390095:
		handleChannelHeartBeat(conn, messageBody)
	case 390094:
		handleResentMsg(conn, messageBody)
	case 390093:
		handleUserReport(conn, messageBody)
	case 390090:
		handleChannelStatistic(conn, messageBody)
	case 390013:
		handleRealtimeStatus(conn, messageBody)
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

func main() {
	conn, err := net.Dial("tcp", "127.0.0.1:6666")
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

		if messageBody == nil {
			fmt.Println("heartbeat message")
			if heartbeat(conn) != nil {
				fmt.Println("heartbeat failed")
			}
		} else {
			handleMessage(conn, msgType, messageBody)
		}
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
