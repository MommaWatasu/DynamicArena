#include <MadgwickAHRS.h>
#include <BleGamepad.h>
#include <Ticker.h>

#define GPIO_PIN_BUTTON 27
#define GPIO_PIN_JOYSTICK_PUSH 34
#define GPIO_PIN_JOYSTICK_X 32
#define GPIO_PIN_JOYSTICK_Y 25

#define DELTA_TIME 10
#define WDT_TIMEOUT 10

//Madgwick MadgwickFilter;
Ticker InputTicker;
TimerHandle_t timer_button = NULL;
BleGamepad bleGamepad("DynamicArenaController", "MommaWatasu", 100);
//float accel_zero[2];

// Button Initializer
void initialize_buttons() {
  pinMode(GPIO_PIN_BUTTON, INPUT_PULLUP);
  pinMode(GPIO_PIN_JOYSTICK_PUSH, INPUT);
  pinMode(GPIO_PIN_JOYSTICK_X, INPUT);
  pinMode(GPIO_PIN_JOYSTICK_Y, INPUT);
}

// check all button states
void check_buttons() {
  if (digitalRead(GPIO_PIN_BUTTON) == LOW) {
    bleGamepad.press(BUTTON_1);
  } else {
    bleGamepad.release(BUTTON_1);
  }
}

void check_joystick() {
  int joystick_x = analogRead(GPIO_PIN_JOYSTICK_X);
  int joystick_y = analogRead(GPIO_PIN_JOYSTICK_Y);
  if (joystick_x < 1000) {
      if (joystick_y < 1000) {
        bleGamepad.setHat(DPAD_DOWN_LEFT);
      } else if (joystick_y > 3000) {
        bleGamepad.setHat(DPAD_UP_LEFT);
      } else {
        bleGamepad.setHat(DPAD_LEFT);
      }
  } else if (joystick_x > 3000) {
    if (joystick_y < 1000) {
      bleGamepad.setHat(DPAD_DOWN_RIGHT);
    } else if (joystick_y > 3000) {
      bleGamepad.setHat(DPAD_UP_RIGHT);
    } else {
      bleGamepad.setHat(DPAD_RIGHT);
    }
  } else {
    if (joystick_y < 1000) {
      bleGamepad.setHat(DPAD_DOWN);
    } else if (joystick_y > 3000) {
      bleGamepad.setHat(DPAD_UP);
    } else {
      bleGamepad.setHat(DPAD_CENTERED);
    }
  }
}

void setup(){
  // Initialize Connections
  Serial.begin(115200);
  while (!Serial) {
    delay(10);
  }

  // Initialize Buttons
  Serial.println("Initializing GPIO INPUT...");
  initialize_buttons();
  Serial.println("Complete");

  // Initialize MadgwickFilter
  /*
  Serial.println("Initializing MadgwickFilter...");
  MadgwickFilter.begin(100);
  MadgwickFilter.setGain(0.8);
  Serial.println("Complete");
  */

  // Initialize BLEGamepad
  Serial.println("Initializing BLEGamepad...");
  bleGamepad.begin();
  Serial.println("Complete");

  // Start Timer Interruption
  Serial.println("Start Timer Interruption");
  InputTicker.attach_ms(DELTA_TIME, pollControllerInput);
}

void pollControllerInput(){
  /*
  MadgwickFilter.updateIMU(gx, gy, gz, ax, ay, az);
  float roll = MadgwickFilter.getRoll();
  float pitch = MadgwickFilter.getPitch();
  float yaw  = MadgwickFilter.getYaw();
  */
  /*
  Serial.print("roll:");Serial.print(roll); Serial.print(",");
  Serial.print("pitch:");Serial.print(pitch); Serial.print(",");
  Serial.print("yaw:");Serial.print(yaw);
  Serial.print("\n");
  */

  if (bleGamepad.isConnected()) {
    // set button state
    check_buttons();
    check_joystick();
    bleGamepad.sendReport();
  }
}

void loop() {}