#include <BleGamepad.h>

// ピン定義 - 実際のピン番号に合わせて変更してください
#define JOYSTICK_NORTH_PIN   4   // ジョイスティック上
#define JOYSTICK_SOUTH_PIN   16   // ジョイスティック下  
#define JOYSTICK_WEST_PIN    17  // ジョイスティック左
#define JOYSTICK_EAST_PIN    5  // ジョイスティック右

#define BUTTON_1_PIN        32  // ボタン1
#define BUTTON_2_PIN        33  // ボタン2
#define BUTTON_3_PIN        25 // ボタン3
#define BUTTON_4_PIN        26  // ボタン4

// BLEGamepadオブジェクト作成
BleGamepad bleGamepad("ESP32ArcadeController", "MommaWatasu", 100);

// 前回の状態を保持する変数
bool prevJoystickUp = false;
bool prevJoystickDown = false;
bool prevJoystickLeft = false;
bool prevJoystickRight = false;

bool prevButton1 = false;
bool prevButton2 = false;
bool prevButton3 = false;
bool prevButton4 = false;

void setup() {
  Serial.begin(115200);
  Serial.println("ESP32 Arcade Controller Starting...");
  
  // ピンモードを入力プルアップに設定（ボタンはGNDに接続）
  pinMode(JOYSTICK_NORTH_PIN, INPUT_PULLUP);
  pinMode(JOYSTICK_SOUTH_PIN, INPUT_PULLUP);
  pinMode(JOYSTICK_WEST_PIN, INPUT_PULLUP);
  pinMode(JOYSTICK_EAST_PIN, INPUT_PULLUP);
  
  pinMode(BUTTON_1_PIN, INPUT_PULLUP);
  pinMode(BUTTON_2_PIN, INPUT_PULLUP);
  pinMode(BUTTON_3_PIN, INPUT_PULLUP);
  pinMode(BUTTON_4_PIN, INPUT_PULLUP);
  
  // BLE Gamepadを開始
  bleGamepad.begin();
  Serial.println("BLE Gamepad initialized. Waiting for connection...");
}

void loop() {
  if (bleGamepad.isConnected()) {
    // ジョイスティック（DPad）の状態読み取り
    bool joystickUp = !digitalRead(JOYSTICK_NORTH_PIN);      // LOW = 押された状態
    bool joystickDown = !digitalRead(JOYSTICK_SOUTH_PIN);
    bool joystickLeft = !digitalRead(JOYSTICK_WEST_PIN);
    bool joystickRight = !digitalRead(JOYSTICK_EAST_PIN);
    
    // ボタンの状態読み取り
    bool button1 = !digitalRead(BUTTON_1_PIN);
    bool button2 = !digitalRead(BUTTON_2_PIN);
    bool button3 = !digitalRead(BUTTON_3_PIN);
    bool button4 = !digitalRead(BUTTON_4_PIN);
    
    // DPadの状態更新（変化があった場合のみ）
    if (joystickUp != prevJoystickUp || joystickDown != prevJoystickDown || 
        joystickLeft != prevJoystickLeft || joystickRight != prevJoystickRight) {
      
      // DPadの値を計算（8方向 + ニュートラル）
      uint8_t dpadValue = DPAD_CENTERED;
      
      if (joystickUp && joystickRight) {
        dpadValue = DPAD_UP_RIGHT;
      } else if (joystickUp && joystickLeft) {
        dpadValue = DPAD_UP_LEFT;
      } else if (joystickDown && joystickRight) {
        dpadValue = DPAD_DOWN_RIGHT;
      } else if (joystickDown && joystickLeft) {
        dpadValue = DPAD_DOWN_LEFT;
      } else if (joystickUp) {
        dpadValue = DPAD_UP;
      } else if (joystickDown) {
        dpadValue = DPAD_DOWN;
      } else if (joystickLeft) {
        dpadValue = DPAD_LEFT;
      } else if (joystickRight) {
        dpadValue = DPAD_RIGHT;
      }
      
      bleGamepad.setHat1(dpadValue);
      
      // デバッグ情報出力
      Serial.print("DPad: ");
      Serial.println(dpadValue);
      
      // 前回の状態を更新
      prevJoystickUp = joystickUp;
      prevJoystickDown = joystickDown;
      prevJoystickLeft = joystickLeft;
      prevJoystickRight = joystickRight;
    }
    
    // ボタン1の状態更新
    if (button1 != prevButton1) {
      if (button1) {
        bleGamepad.press(BUTTON_1);
        Serial.println("Button 1 pressed");
      } else {
        bleGamepad.release(BUTTON_1);
        Serial.println("Button 1 released");
      }
      prevButton1 = button1;
    }
    
    // ボタン2の状態更新
    if (button2 != prevButton2) {
      if (button2) {
        bleGamepad.press(BUTTON_2);
        Serial.println("Button 2 pressed");
      } else {
        bleGamepad.release(BUTTON_2);
        Serial.println("Button 2 released");
      }
      prevButton2 = button2;
    }
    
    // ボタン3の状態更新
    if (button3 != prevButton3) {
      if (button3) {
        bleGamepad.press(BUTTON_5);
        Serial.println("Button 3 pressed");
      } else {
        bleGamepad.release(BUTTON_5);
        Serial.println("Button 3 released");
      }
      prevButton3 = button3;
    }
    
    // ボタン4の状態更新
    if (button4 != prevButton4) {
      if (button4) {
        bleGamepad.press(BUTTON_4);
        Serial.println("Button 4 pressed");
      } else {
        bleGamepad.release(BUTTON_4);
        Serial.println("Button 4 released");
      }
      prevButton4 = button4;
    }
  } else {
    Serial.println("Waiting for BLE connection...");
  }
  
  // チャタリング防止のための短い遅延
  delay(10);
}