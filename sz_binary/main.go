package main

import (
	"bytes"
	"encoding/binary"
	"errors"
	"fmt"
	"io"
	"net"
	"os"
)

//MsgHead defines a
//generic head structure
type MsgHead struct {
	_msgType    uint32
	_bodyLength uint32
}

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

	if bodyLen > 0 {
		buf := make([]byte, bodyLen)

		n, err2 := conn.Read(buf)
		if err2 != nil {
			return 0, 0, nil, err2
		}

		var checksum uint32
		err2 = binary.Read(conn, binary.BigEndian, &checksum)
		if err2 != nil {
			return 0, 0, nil, err2
		}

		if n != int(bodyLen) {
			return msgType, checksum, nil, errors.New("Wrong size expect")
		}

		return msgType, checksum, buf, nil
	}

	return msgType, 0, nil, errors.New("Wrong msg")
}

func handleChannelHeartBeat(conn net.Conn, messageBody []byte) {

}

func handleResentMsg(conn net.Conn, messageBody []byte) {

}

func handleUserReport(conn net.Conn, messageBody[]byte)
{
    
}

func handleMessage(conn net.Conn, msgType uint32, messageBody []byte) {

	switch msgType {
	case 390095:
		handleChannelHeartBeat(conn, messageBody)
	case 390094:
		handleResentMsg(conn, messageBody)
	case 390093:
		handleUserReport(conn, messageBody)
	default:
		fmt.Println(msgType, messageBody)
	}

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

	for {
		msgType, _, messageBody, err2 := getMessage(conn)
		if err2 != nil {
			fmt.Println(err2)
			break
		}

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
