#include <Adafruit_MPU6050.h>
#include <MadgwickAHRS.h>
#include <BleGamepad.h>
#include <Ticker.h>

#define GPIO_PIN_A 19

#define DELTA_TIME 10
#define WDT_TIMEOUT 10

Adafruit_MPU6050 mpu;
Madgwick MadgwickFilter;
Ticker InputTicker;
TimerHandle_t timer_button_a = NULL;
BleGamepad bleGamepad("DynamicArena Controller", "MommaWatasu", 100);
float accel_zero[2];

// Button Initializer
void initialize_buttons() {
  pinMode(GPIO_PIN_A, INPUT_PULLUP);
}

// check all button states
void check_buttons() {
  if (digitalRead(GPIO_PIN_A) == HIGH) {
    bleGamepad.press(BUTTON_1);
  } else {
    bleGamepad.release(BUTTON_1);
  }
}

void setup(){
  // Initialize Connections
  Wire.begin();
  Serial.begin(115200);
  while (!Serial) {
    delay(10);
  }

  // Initialize MPU6050
  Serial.println("Initializing MPU6050...");
  // start up
  if (!mpu.begin()) {
    Serial.println("Failed to initialize MPU6050. Please check out the connection...");
    while (1) {
      delay(10);
    }
  }
  // configuration
  mpu.setAccelerometerRange(MPU6050_RANGE_2_G);
  mpu.setGyroRange(MPU6050_RANGE_250_DEG);
  mpu.setFilterBandwidth(MPU6050_BAND_21_HZ);
  Serial.println("Complete");

  // Initialize Buttons
  Serial.println("Initializing Buttons...");
  initialize_buttons();
  Serial.println("Complete");

  // Initialize MadgwickFilter
  Serial.println("Initializing MadgwickFilter...");
  MadgwickFilter.begin(100);
  MadgwickFilter.setGain(0.8);
  Serial.println("Complete");

  // Initialize BLEGamepad
  Serial.println("Initializing BLEGamepad...");
  bleGamepad.begin();
  Serial.println("Complete");

  // Start Timer Interruption
  Serial.println("Start Timer Interruption");
  InputTicker.attach_ms(DELTA_TIME, pollControllerInput);
}

void pollControllerInput(){
  sensors_event_t a, g, temp;
  mpu.getEvent(&a, &g, &temp);

  // acceleration
  float ax = a.acceleration.x - accel_zero[0];
  float ay = a.acceleration.y - accel_zero[1];
  float az = a.acceleration.z;
  // gyro
  float gx = g.gyro.x;
  float gy = g.gyro.y;
  float gz = g.gyro.z;
  MadgwickFilter.updateIMU(gx, gy, gz, ax, ay, az);
  float roll = MadgwickFilter.getRoll();
  float pitch = MadgwickFilter.getPitch();
  float yaw  = MadgwickFilter.getYaw();
  /*
  Serial.print("roll:");Serial.print(roll); Serial.print(",");
  Serial.print("pitch:");Serial.print(pitch); Serial.print(",");
  Serial.print("yaw:");Serial.print(yaw);
  Serial.print("\n");
  */

  if (bleGamepad.isConnected()) {
    // set accel data
    bleGamepad.setAccelerometer(ax, ay, az);
    // set gyro data
    bleGamepad.setGyroscope(roll, pitch, yaw);
    // set button state
    check_buttons();
    bleGamepad.sendReport();
  }
}

void loop() {}