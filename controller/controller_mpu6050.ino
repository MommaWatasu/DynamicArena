#include <Arduino.h>
#include <Wire.h>
#include <BluetoothSerial.h>

#define MPU6050_ADDR         0x68
#define MPU6050_SMPLRT_DIV   0x19
#define MPU6050_CONFIG       0x1a
#define MPU6050_GYRO_CONFIG  0x1b
#define MPU6050_ACCEL_CONFIG 0x1c
#define MPU6050_WHO_AM_I     0x75
#define MPU6050_PWR_MGMT_1   0x6b

// How often to send data (in milliseconds)
#define SEND_INTERVAL_MS     50   // 20Hz

double offsetX = 0, offsetY = 0, offsetZ = 0;
double gyro_angle_x = 0, gyro_angle_y = 0, gyro_angle_z = 0;
float angleX, angleY, angleZ;
float interval, preInterval;
float acc_x, acc_y, acc_z, acc_angle_x, acc_angle_y;
float dpsX, dpsY, dpsZ;

// Function declarations
int myFunction(int, int);
void writeMPU6050(byte reg, byte data);
byte readMPU6050(byte reg);
void calcRotation();

BluetoothSerial SerialBT;
static const int BUTTON_PIN   = 12;
static const int JOY_X_PIN    = 34;
static const int JOY_Y_PIN    = 35;
static const int JOY_BTN_PIN  = 14;

void setup() {
  // Dummy computation
  int result = myFunction(2, 3);

  // Initialize I2C and Bluetooth
  Wire.begin(26, 25);
  SerialBT.begin("ESP32_MPU_Controller");

  // Configure inputs
  pinMode(BUTTON_PIN, INPUT_PULLUP);
  pinMode(JOY_BTN_PIN, INPUT_PULLUP);

  // Verify MPU6050 presence
  if (readMPU6050(MPU6050_WHO_AM_I) != 0x68) {
    SerialBT.println("WHO_AM_I error.");
    while (true) {}
  }

  // Basic sensor setup
  writeMPU6050(MPU6050_SMPLRT_DIV, 0x00);
  writeMPU6050(MPU6050_CONFIG,    0x00);
  writeMPU6050(MPU6050_GYRO_CONFIG,0x08);
  writeMPU6050(MPU6050_ACCEL_CONFIG,0x00);
  writeMPU6050(MPU6050_PWR_MGMT_1, 0x01);

  // Gyro calibration
  for (int i = 0; i < 3000; i++) {
    Wire.beginTransmission(MPU6050_ADDR);
    Wire.write(0x3B);
    Wire.endTransmission(false);
    Wire.requestFrom(MPU6050_ADDR, 14, true);

    int16_t raw_gyro_x = (Wire.read() << 8 | Wire.read());
    int16_t raw_gyro_y = (Wire.read() << 8 | Wire.read());
    int16_t raw_gyro_z = (Wire.read() << 8 | Wire.read());

    dpsX = raw_gyro_x / 65.5;
    dpsY = raw_gyro_y / 65.5;
    dpsZ = raw_gyro_z / 65.5;
    offsetX += dpsX;
    offsetY += dpsY;
    offsetZ += dpsZ;
  }

  offsetX /= 3000;
  offsetY /= 3000;
  offsetZ /= 3000;

  // Initialize timing
  preInterval = millis();
}

void loop() {
  // Update rotation angles
  calcRotation();

  // Read input states
  int buttonState = digitalRead(BUTTON_PIN);
  int joyX        = analogRead(JOY_X_PIN);
  int joyY        = analogRead(JOY_Y_PIN);
  int joyBtnState = digitalRead(JOY_BTN_PIN);

  // Send formatted data over Bluetooth
  SerialBT.printf("AngleX:%.2f, AngleY:%.2f, AngleZ:%.2f, BTN:%d, JOY:%d,%d,%d\n",
                  angleX, angleY, angleZ,
                  buttonState, joyX, joyY, joyBtnState);

  // Wait before next send
  delay(SEND_INTERVAL_MS);
}

int myFunction(int x, int y) {
  return x + y;
}

void writeMPU6050(byte reg, byte data) {
  Wire.beginTransmission(MPU6050_ADDR);
  Wire.write(reg);
  Wire.write(data);
  Wire.endTransmission();
}

byte readMPU6050(byte reg) {
  Wire.beginTransmission(MPU6050_ADDR);
  Wire.write(reg);
  Wire.endTransmission(true);
  Wire.requestFrom(MPU6050_ADDR, 1);
  return Wire.read();
}

void calcRotation() {
  Wire.beginTransmission(MPU6050_ADDR);
  Wire.write(0x3B);
  Wire.endTransmission(false);
  Wire.requestFrom(MPU6050_ADDR, 14, true);

  int16_t raw_acc_x = (Wire.read() << 8 | Wire.read());
  int16_t raw_acc_y = (Wire.read() << 8 | Wire.read());
  int16_t raw_acc_z = (Wire.read() << 8 | Wire.read());
  Wire.read(); Wire.read(); // skip temp
  int16_t raw_gyro_x = (Wire.read() << 8 | Wire.read());
  int16_t raw_gyro_y = (Wire.read() << 8 | Wire.read());
  int16_t raw_gyro_z = (Wire.read() << 8 | Wire.read());

  // Convert accelerations to angles
  acc_x = raw_acc_x / 16384.0;
  acc_y = raw_acc_y / 16384.0;
  acc_z = raw_acc_z / 16384.0;
  acc_angle_y = atan2(acc_x, acc_z + abs(acc_y)) * 360 / -2.0 / PI;
  acc_angle_x = atan2(acc_y, acc_z + abs(acc_x)) * 360 / 2.0 / PI;

  // Convert gyro readings and integrate
  dpsX = raw_gyro_x / 65.5;
  dpsY = raw_gyro_y / 65.5;
  dpsZ = raw_gyro_z / 65.5;
  interval = millis() - preInterval;
  preInterval = millis();
  gyro_angle_x += (dpsX - offsetX) * (interval * 0.001);
  gyro_angle_y += (dpsY - offsetY) * (interval * 0.001);
  gyro_angle_z += (dpsZ - offsetZ) * (interval * 0.001);

  // Complementary filter
  angleX = (0.996 * gyro_angle_x) + (0.004 * acc_angle_x);
  angleY = (0.996 * gyro_angle_y) + (0.004 * acc_angle_y);
  angleZ = gyro_angle_z;

  gyro_angle_x = angleX;
  gyro_angle_y = angleY;
  gyro_angle_z = angleZ;
}
